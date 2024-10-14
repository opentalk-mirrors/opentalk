// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::sip::{SdpRecvCodec, SdpSendCodec, SrtpInfo};
use anyhow::{Context, Result};
use codec::Codec;
use dtmf_filter::DtmfFilter;
use ezk::{
    nodes::{Access, AccessHandle},
    ConfigRange, Frame, NextEventIsCancelSafe, Source, SourceEvent, ValueRange,
};
use ezk_audio::{
    Channels, Format, RawAudio, RawAudioConfig, RawAudioConfigRange, SampleRate, Samples,
};
use ezk_audio_nodes::{AudioConvert, AudioMixer};
use ezk_g711::{PCMADecoder, PCMAEncoder, PCMUDecoder, PCMUEncoder};
use ezk_rtp::{DePacketizer, Packetizer};
use opentalk_compositor::{
    audio::Silence,
    livekit::{
        self,
        options::TrackPublishOptions,
        track::{LocalAudioTrack, LocalTrack},
        webrtc::{
            audio_source::native::NativeAudioSource,
            prelude::{AudioFrame, AudioSourceOptions, RtcAudioSource},
        },
    },
    Mixer,
};
use rtp::{RtpMpscSource, RtpSessions};
use std::{collections::HashMap, future::Future, net::SocketAddr, sync::Arc};
use tokio::{
    net::UdpSocket,
    sync::{mpsc, Mutex},
    task::JoinHandle,
};

pub mod codec;
mod dtmf_filter;
mod port_pool;
mod rtp;
mod track;

pub(crate) use port_pool::{PortPool, SocketPair};
pub(crate) use track::{Track, TrackController, TrackSource};

const RAW_AUDIO_CONFIG_RANGE: RawAudioConfigRange = RawAudioConfigRange {
    sample_rate: ValueRange::Value(SampleRate(48000)),
    channels: ValueRange::Value(Channels::NotPositioned(2)),
    format: ValueRange::Value(ezk_audio::Format::I16),
};
const RAW_AUDIO_CONFIG: RawAudioConfig = RawAudioConfig {
    sample_rate: SampleRate(48000),
    channels: Channels::NotPositioned(2),
    format: Format::I16,
};

/// Asynchronous events emitted by the media pipeline
pub enum MediaEvent {
    Error,
    DtmfDigit(u8),
}

pub struct MediaPipeline {
    pub compositor: Option<Mixer>,

    rtp_sessions: RtpSessions,

    subscriber_audio_mixer: AccessHandle<AudioMixer>,
    subscriber_audio_tx: Arc<Mutex<HashMap<u32, mpsc::Sender<Frame<RawAudio>>>>>,
    subscriber_audio_mixer_task: JoinHandle<()>,

    publish_audio_mixer: AccessHandle<AudioMixer>,
    publish_audio_tx: Arc<Mutex<Option<NativeAudioSource>>>,
    publish_audio_mixer_task: JoinHandle<()>,

    pub(crate) events_tx: mpsc::UnboundedSender<MediaEvent>,
    events_rx: mpsc::UnboundedReceiver<MediaEvent>,
}

impl MediaPipeline {
    /// # Returns
    ///
    /// - Self
    /// - track controller to play different audio tracks
    pub async fn new() -> Result<(Self, TrackController)> {
        let (events_tx, events_rx) = mpsc::unbounded_channel();

        // [compositor + ] obelisk -> phone
        let track_source = TrackSource::new();
        let track_controller = track_source.controller();

        let (subscriber_audio_mixer, subscriber_audio_mixer_access_handle) =
            Access::new(AudioMixer::new(track_source));

        let subscriber_audio_tx: Arc<Mutex<HashMap<u32, mpsc::Sender<Frame<RawAudio>>>>> =
            Arc::default();
        let subscriber_audio_tx_ = subscriber_audio_tx.clone();
        let subscriber_audio_mixer_task =
            tokio::spawn(audio_mixer_task(subscriber_audio_mixer, move |frame| {
                let x = subscriber_audio_tx_.clone();

                async move {
                    for tx in x.lock().await.values() {
                        if let Err(e) = tx.send(frame.clone()).await {
                            log::error!("Failed to forward subscriber audio frame, {e}");
                        }
                    }
                }
            }));

        // phone -> obelisk -> conference
        let (publish_audio_mixer, publish_audio_mixer_access_handle) =
            Access::new(AudioMixer::new(Silence::default()));
        let publish_audio_tx: Arc<Mutex<Option<NativeAudioSource>>> = Arc::default();
        let publish_audio_tx_ = publish_audio_tx.clone();
        let publish_audio_mixer_task = tokio::spawn(audio_mixer_task(
            publish_audio_mixer,
            move |frame| {
                let publish_audio_tx_ = publish_audio_tx_.clone();

                async move {
                    if let Some(source) = &*publish_audio_tx_.lock().await {
                        let Samples::I16(samples) = frame.into_data().samples else {
                            unreachable!("Cannot receive non-i16 audio samples in audio_mixer_task's on_frame")
                        };

                        let frame = AudioFrame {
                            samples_per_channel: (samples.len() / 2) as u32,
                            data: samples.into(),
                            sample_rate: 48_000,
                            num_channels: 2,
                        };

                        if let Err(e) = source.capture_frame(&frame).await {
                            log::error!("Failed to send audio frame to livekit, {e}");
                        }
                    }
                }
            },
        ));

        let rtp_sessions = RtpSessions::new({
            let events_tx = events_tx.clone();

            move || {
                let _ = events_tx.send(MediaEvent::Error);
            }
        });

        let this = Self {
            compositor: None,

            rtp_sessions,

            subscriber_audio_mixer: subscriber_audio_mixer_access_handle,
            subscriber_audio_tx,
            subscriber_audio_mixer_task,

            publish_audio_mixer: publish_audio_mixer_access_handle,
            publish_audio_tx,
            publish_audio_mixer_task,

            events_tx,
            events_rx,
        };

        Ok((this, track_controller))
    }

    pub async fn add_compositor(&mut self, mut compositor: Mixer) -> Result<()> {
        // Setup subscription
        let (audio_tx, audio_rx) = mpsc::channel(8);

        compositor
            .link_sink(
                "obelisk",
                Box::new(CompositorObeliskBridgeSink { audio_tx }),
            )
            .await
            .context("Failed to add obelisk sink to compositor")?;

        self.subscriber_audio_mixer
            .access(|audio_mixer| {
                audio_mixer.add_source(AudioMpscSource { rx: audio_rx });
            })
            .await
            .context("Failed to add compositor's audio stream to subscriber_audio_mixer")?;

        // Setup publish
        let local_participant = compositor.local_participant();

        let audio_source = NativeAudioSource::new(AudioSourceOptions::default(), 48_000, 2, 100);
        let local_audio_track = LocalAudioTrack::create_audio_track(
            "audio",
            RtcAudioSource::Native(audio_source.clone()),
        );

        local_participant
            .publish_track(
                LocalTrack::Audio(local_audio_track.clone()),
                TrackPublishOptions {
                    source: livekit::track::TrackSource::Microphone,
                    ..Default::default()
                },
            )
            .await
            .context("Failed to publish audio track")?;

        *self.publish_audio_tx.lock().await = Some(audio_source);

        self.compositor = Some(compositor);

        Ok(())
    }

    pub async fn wait_for_event(&mut self) -> MediaEvent {
        if let Some(compositor) = &mut self.compositor {
            tokio::select! {
                event = self.events_rx.recv() => {
                    event.expect("self also holds the sender side of the channel, so this should never fail")
                }
                result = compositor.run() => {
                    match result {
                        Ok(()) => unreachable!("compositor's run function never returns unless an error occurs"),
                        Err(e) => {
                            log::error!("compositor run failed with, {e}");
                            MediaEvent::Error
                        }
                    }
                }
            }
        } else {
            self.events_rx
                .recv()
                .await
                .expect("self also holds the sender side of the channel, so this should never fail")
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_sender(
        &mut self,
        session_id: u32,
        srtp: Option<&SrtpInfo>,
        codec: &SdpSendCodec,
        rtp_socket: Arc<UdpSocket>,
        rtcp_socket: Arc<UdpSocket>,
        remote_rtp_addr: SocketAddr,
        remote_rtcp_addr: SocketAddr,
    ) -> Result<()> {
        log::trace!("add sender session_id={session_id} {codec:?}");

        let source = match codec.codec.media_type() {
            codec::MediaType::Audio => {
                let (tx, rx) = mpsc::channel(8);

                self.subscriber_audio_tx.lock().await.insert(session_id, tx);

                let audio_source = AudioConvert::new(AudioMpscSource { rx });

                match codec.codec {
                    Codec::G711PCMA => {
                        Packetizer::new(PCMAEncoder::new(audio_source)).boxed_cancel_safe()
                    }
                    Codec::G711PCMU => {
                        Packetizer::new(PCMUEncoder::new(audio_source)).boxed_cancel_safe()
                    }
                    Codec::H264 => unreachable!(),
                }
            }
            codec::MediaType::Video => unimplemented!("video support is not yet implemented"),
        };

        self.rtp_sessions
            .add_sender(
                session_id,
                srtp,
                codec.codec.clock_rate(),
                codec.payload,
                rtp_socket,
                rtcp_socket,
                remote_rtp_addr,
                remote_rtcp_addr,
                source,
            )
            .await
            .context("Failed to add sender to rtp_sessions")
    }

    pub async fn remove_sender(&mut self, session_id: u32) -> Result<()> {
        log::debug!("remove sender session_id={session_id}");

        // TODO: also remove video tx

        self.subscriber_audio_tx.lock().await.remove(&session_id);
        self.rtp_sessions
            .remove_sender(session_id)
            .await
            .context("Failed to remove sender from rtp-sessions")
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_receiver(
        &mut self,
        session_id: u32,
        srtp: Option<&SrtpInfo>,
        codec: &SdpRecvCodec,
        dtmf: Option<(u8, mpsc::UnboundedSender<MediaEvent>)>,
        rtp_socket: Arc<UdpSocket>,
        rtcp_socket: Arc<UdpSocket>,
        remote_rtp_addr: SocketAddr,
        remote_rtcp_addr: SocketAddr,
    ) -> Result<()> {
        log::debug!("add receiver session_id={session_id} {codec:?}");

        let rtp_source = self
            .rtp_sessions
            .add_receiver(
                session_id,
                srtp,
                codec.codec.clock_rate(),
                codec.payload,
                rtp_socket,
                rtcp_socket,
                remote_rtp_addr,
                remote_rtcp_addr,
            )
            .await
            .context("Failed to add receiver to rtp-sessions")?;

        let rtp_source = DtmfFilter {
            source: rtp_source,
            media_pt: codec.payload,
            dtmf,
            last_dtmf_timestamp: None,
        };

        let audio_source = match codec.codec {
            Codec::G711PCMA => PCMADecoder::new(DePacketizer::new(rtp_source)).boxed_cancel_safe(),
            Codec::G711PCMU => PCMUDecoder::new(DePacketizer::new(rtp_source)).boxed_cancel_safe(),
            Codec::H264 => unimplemented!("video is not yet supported"),
        };

        self.publish_audio_mixer
            .access(|mixer| {
                mixer.add_source(AudioConvert::new(audio_source));
            })
            .await
            .context("Failed to add receiver to audio mixer")
    }

    pub async fn remove_receiver(&mut self, session_id: u32) -> Result<()> {
        log::debug!("remove receiver session_id={session_id}");

        self.rtp_sessions
            .remove_receiver(session_id)
            .await
            .context("Failed to remove sender from rtp-sessions")
    }

    pub fn post_update_cleanup(&mut self) {
        log::debug!("run post update cleanup");

        self.rtp_sessions
            .cleanup_sessions_without_sender_and_receiver();
    }
}

impl Drop for MediaPipeline {
    fn drop(&mut self) {
        self.publish_audio_mixer_task.abort();
        self.subscriber_audio_mixer_task.abort();
    }
}

async fn audio_mixer_task<Fn, Fut>(audio_mixer: Access<AudioMixer>, on_frame: Fn)
where
    Fn: FnMut(Frame<RawAudio>) -> Fut,
    Fut: Future<Output = ()>,
{
    if let Err(e) = audio_mixer_task_inner(audio_mixer, on_frame).await {
        log::error!("audio mixer task failed, {e:?}");
    }
}

async fn audio_mixer_task_inner<Fn, Fut>(
    mut audio_mixer: Access<AudioMixer>,
    mut on_frame: Fn,
) -> Result<()>
where
    Fn: FnMut(Frame<RawAudio>) -> Fut,
    Fut: Future<Output = ()>,
{
    'negotiate: loop {
        audio_mixer
            .negotiate_config(vec![RAW_AUDIO_CONFIG_RANGE])
            .await
            .context("Failed to negotiate config")?;

        loop {
            let event = audio_mixer
                .next_event()
                .await
                .context("unable to poll next event")?;

            match event {
                SourceEvent::Frame(frame) => on_frame(frame).await,
                SourceEvent::EndOfData => {
                    // No more audio sources, this task quits now.
                    log::debug!("audio mixer task exiting");
                    return Ok(());
                }
                SourceEvent::RenegotiationNeeded => {
                    continue 'negotiate;
                }
            }
        }
    }
}

#[derive(Debug)]
struct CompositorObeliskBridgeSink {
    audio_tx: mpsc::Sender<Frame<RawAudio>>,
}

impl opentalk_compositor::Sink for CompositorObeliskBridgeSink {
    fn on_audio_frame(&mut self, frame: Frame<RawAudio>) -> Result<()> {
        if let Err(e) = self.audio_tx.try_send(frame) {
            log::debug!("Failed to send audio sample from compositor to audiomixer, {e}");
        }

        Ok(())
    }

    fn on_video_frame(&mut self, _buffer: Vec<u8>) -> Result<()> {
        Ok(())
    }
}

struct AudioMpscSource {
    rx: mpsc::Receiver<Frame<RawAudio>>,
}

impl Source for AudioMpscSource {
    type MediaType = RawAudio;

    async fn capabilities(&mut self) -> ezk::Result<Vec<RawAudioConfigRange>> {
        Ok(vec![RAW_AUDIO_CONFIG_RANGE])
    }

    async fn negotiate_config(
        &mut self,
        available: Vec<RawAudioConfigRange>,
    ) -> ezk::Result<RawAudioConfig> {
        assert!(available.iter().any(|r| r.contains(&RAW_AUDIO_CONFIG)));
        Ok(RAW_AUDIO_CONFIG)
    }

    async fn next_event(&mut self) -> ezk::Result<SourceEvent<RawAudio>> {
        if let Some(frame) = self.rx.recv().await {
            Ok(SourceEvent::Frame(frame))
        } else {
            Ok(SourceEvent::EndOfData)
        }
    }
}

impl NextEventIsCancelSafe for AudioMpscSource {}

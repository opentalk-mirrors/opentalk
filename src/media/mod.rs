// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{
    settings::Settings,
    sip::{MediaSessionId, SdpRecvCodec, SdpSendCodec, SdpSendCodecParams, SrtpInfo},
};
use anyhow::{Context, Result};
use codec::Codec;
use core::fmt;
use dtmf_filter::DtmfFilter;
use ezk::{
    nodes::{Access, AccessHandle},
    BoxedSourceCancelSafe, ConfigRange, Frame, NextEventIsCancelSafe, Source, SourceEvent,
    ValueRange,
};
use ezk_audio::{
    Channels, Format, RawAudio, RawAudioConfig, RawAudioConfigRange, SampleRate, Samples,
};
use ezk_audio_nodes::{AudioConvert, AudioMixer};
use ezk_g711::{PCMADecoder, PCMAEncoder, PCMUDecoder, PCMUEncoder};
use ezk_image::{
    resize::{FilterType, ResizeAlg, Resizer},
    Image, PixelFormat, PixelFormatPlanes,
};
use ezk_rtp::{rtp_types::RtpPacketBuilder, DePacketizer, Packetizer, RtpPacket};
use opentalk_compositor::{
    audio::Silence,
    livekit::{
        self,
        options::TrackPublishOptions,
        prelude::*,
        webrtc::{
            audio_source::native::NativeAudioSource,
            prelude::{
                AudioFrame, AudioSourceOptions, RtcAudioSource, RtcVideoSource, VideoResolution,
            },
            video_source::native::NativeVideoSource,
        },
    },
    Mixer, HEIGHT, WIDTH,
};
use rand::Rng;
use rtp::{RtpMpscSource, RtpSessions};
use std::{
    collections::HashMap,
    future::Future,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    net::UdpSocket,
    sync::{mpsc, Mutex, RwLock},
    task::JoinHandle,
};
use video::h264::{H264Decoder, H264Encoder};

pub mod codec;
mod dtmf_filter;
mod port_pool;
mod rtp;
mod track;
mod video;

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

pub type VideoSinks = Arc<Mutex<HashMap<MediaSessionId, Box<dyn FnMut(&[u8]) + Send>>>>;

#[allow(clippy::type_complexity)]
pub struct MediaPipeline {
    settings: Arc<Settings>,

    pub compositor: Option<Mixer>,

    rtp_sessions: RtpSessions,

    subscriber_audio_mixer: AccessHandle<AudioMixer>,
    subscriber_audio_tx: Arc<Mutex<HashMap<MediaSessionId, mpsc::Sender<Frame<RawAudio>>>>>,
    subscriber_audio_mixer_task: JoinHandle<()>,

    // pre-join UI controller
    pub(super) ui_controller: video::ui::UiController,

    // obelisk -> phone video sinks. One sink for every video sender.
    subscriber_video_tx: VideoSinks,

    // maps RTP receiver to livekit publish video sources
    // This publishes a separate livekit track for every incoming (phone -> obelisk) video stream
    publish_video_tx: Arc<RwLock<HashMap<MediaSessionId, Option<(TrackSid, NativeVideoSource)>>>>,

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
    pub async fn new(settings: Arc<Settings>) -> Result<(Self, TrackController)> {
        let (events_tx, events_rx) = mpsc::unbounded_channel();

        // [compositor + ] obelisk -> phone
        let track_source = TrackSource::new();
        let track_controller = track_source.controller();

        let (subscriber_audio_mixer, subscriber_audio_mixer_access_handle) =
            Access::new(AudioMixer::new(track_source));

        let subscriber_audio_tx: Arc<
            Mutex<HashMap<MediaSessionId, mpsc::Sender<Frame<RawAudio>>>>,
        > = Arc::default();
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

        let video_tx = VideoSinks::default();
        let ui_controller = video::ui::spawn_ui_thread(video_tx.clone());

        let this = Self {
            settings,

            compositor: None,

            rtp_sessions,

            subscriber_audio_mixer: subscriber_audio_mixer_access_handle,
            subscriber_audio_tx,
            subscriber_audio_mixer_task,

            ui_controller,

            subscriber_video_tx: video_tx,
            publish_video_tx: Arc::default(),

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

        // we're about to join the conference, stop the UI thread
        self.ui_controller.stop();

        compositor
            .link_sink(
                "obelisk",
                Box::new(CompositorObeliskBridgeSink {
                    audio_tx,
                    video_tx: self.subscriber_video_tx.clone(),
                }),
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

        // Audio
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

        // Video
        let mut publish_video_tx = self.publish_video_tx.write().await;
        for publish_video_source in publish_video_tx.values_mut() {
            let video_source = livekit_publish_video(&local_participant).await?;

            *publish_video_source = Some(video_source);
        }

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
        session_id: MediaSessionId,
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
            codec::MediaType::Video => self.create_h264_sender(session_id, codec).await?,
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

    async fn create_h264_sender(
        &mut self,
        session_id: MediaSessionId,
        codec: &SdpSendCodec,
    ) -> Result<BoxedSourceCancelSafe<ezk_rtp::Rtp>> {
        assert_eq!(codec.codec, Codec::H264);

        let (tx, rx) = mpsc::channel(8);

        // Resizer to fit the image to the correct dimensions
        let mut resizer =
            ezk_image::resize::Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
        let (target_width, target_height) = match codec.codec_params {
            SdpSendCodecParams::None => (WIDTH, HEIGHT),
            SdpSendCodecParams::H264 {
                width,
                height,
                fps: _,     // TODO: fps limiting
                profile: _, // TODO: h264 profiles
            } => (width as usize, height as usize),
        };
        let mut staging = vec![0u8; PixelFormat::I420.buffer_size(target_width, target_height)];

        let max_bitrate = self.settings.sip.max_video_bitrate;
        let bitrate = codec.bitrate.unwrap_or(max_bitrate).min(max_bitrate);
        let mut h264_enc = H264Encoder::create(bitrate)?;

        // With a clock-rate of 90_000 a u32 timestamp can cover 47721 seconds of time, so generate a random point in that timestamp range
        let start_timestamp =
            Instant::now() - Duration::from_secs(rand::thread_rng().gen_range(0..45000));
        let payload_type = codec.payload;
        let mut sequence_number = rand::random::<u16>() / 2;

        #[allow(clippy::type_complexity)]
        let on_video_frame: Box<dyn FnMut(&[u8]) + Send + 'static> = Box::new(move |buf| {
            let buf = maybe_resize(&mut resizer, buf, &mut staging, target_width, target_height);

            let packets = h264_enc.encode_and_packetize(buf, target_width, target_height);

            // Calculate the timestamp from the seconds passed since origin * clock-rate (90_000) and use only the lower 32bit
            let timestamp = (((start_timestamp.elapsed().as_secs_f64() * 90_000.0) as u64)
                & u64::from(u32::MAX)) as u32;

            for (i, packet) in packets.iter().enumerate() {
                // Set the marker bit for the last packet
                let marker = i == (packets.len() - 1);

                let _ = tx.blocking_send(RtpPacket::new(
                    &RtpPacketBuilder::new()
                        .timestamp(timestamp)
                        .sequence_number(sequence_number)
                        .marker_bit(marker)
                        .payload(packet.as_ref())
                        .payload_type(payload_type),
                ));

                sequence_number = sequence_number.wrapping_add(1);
            }
        });

        self.subscriber_video_tx
            .lock()
            .await
            .insert(session_id, on_video_frame);

        Ok(RtpMpscSource {
            rx,
            pt: codec.payload,
        }
        .boxed_cancel_safe())
    }

    pub async fn remove_sender(&mut self, session_id: MediaSessionId) -> Result<()> {
        log::debug!("remove sender session_id={session_id}");

        self.subscriber_video_tx.lock().await.remove(&session_id);
        self.subscriber_audio_tx.lock().await.remove(&session_id);

        self.rtp_sessions
            .remove_sender(session_id)
            .await
            .context("Failed to remove sender from rtp-sessions")
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_receiver(
        &mut self,
        session_id: MediaSessionId,
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

        match codec.codec {
            Codec::G711PCMA => {
                self.add_audio_receiver(
                    PCMADecoder::new(DePacketizer::new(rtp_source)).boxed_cancel_safe(),
                )
                .await
            }
            Codec::G711PCMU => {
                self.add_audio_receiver(
                    PCMUDecoder::new(DePacketizer::new(rtp_source)).boxed_cancel_safe(),
                )
                .await
            }
            Codec::H264 => self
                .create_h264_receiver(session_id, rtp_source)
                .await
                .context("Failed to add H.264 receiver"),
        }
    }

    async fn create_h264_receiver(
        &mut self,
        session_id: MediaSessionId,
        mut rtp_source: DtmfFilter,
    ) -> Result<()> {
        let mut video_sources = self.publish_video_tx.write().await;

        if let Some(compositor) = &mut self.compositor {
            video_sources.insert(
                session_id,
                Some(
                    livekit_publish_video(&compositor.local_participant())
                        .await
                        .context("Failed to publish video track to livekit")?,
                ),
            );
        } else {
            // No compositor yet, source will be added later in `add_compositor`
            video_sources.insert(session_id, None);
        }

        let mut h264_dec = H264Decoder::create()?;
        let video_sources = self.publish_video_tx.clone();

        tokio::spawn(async move {
            loop {
                let event = match rtp_source.next_event().await {
                    Ok(event) => event,
                    Err(e) => {
                        log::error!("H.264 forward task failed with error, {e}");
                        return;
                    }
                };

                match event {
                    SourceEvent::Frame(frame) => {
                        let video_sources = video_sources.read().await;

                        // Decode the frame, then check if there's a livekit publication yet
                        if let Some(frame) = h264_dec.decode_packet(frame.data().get().payload()) {
                            if let Some((_, video_source)) =
                                video_sources.get(&session_id).and_then(|s| s.as_ref())
                            {
                                video_source.capture_frame(&frame);
                            }
                        }
                    }
                    SourceEvent::EndOfData => {
                        log::debug!("exit H.264 forward task");
                        return;
                    }
                    SourceEvent::RenegotiationNeeded => {
                        unreachable!("RTP receiver sources should never need renegotiation")
                    }
                }
            }
        });

        Ok(())
    }

    async fn add_audio_receiver(
        &mut self,
        audio_source: BoxedSourceCancelSafe<RawAudio>,
    ) -> Result<()> {
        self.publish_audio_mixer
            .access(|mixer| {
                mixer.add_source(AudioConvert::new(audio_source));
            })
            .await
            .context("Failed to add receiver to audio mixer")
    }

    pub async fn remove_receiver(&mut self, session_id: MediaSessionId) -> Result<()> {
        log::debug!("remove receiver session_id={session_id}");

        if let Some((track_id, _)) = self
            .publish_video_tx
            .write()
            .await
            .remove(&session_id)
            .flatten()
        {
            let compositor = &self
                .compositor
                .as_mut()
                .expect("compositor must be set when publishing");

            if let Err(e) = compositor
                .local_participant()
                .unpublish_track(&track_id)
                .await
            {
                log::warn!("Failed to unpublish track {track_id:?}, {e}")
            }
        }

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

async fn livekit_publish_video(
    local_participant: &LocalParticipant,
) -> Result<(TrackSid, NativeVideoSource)> {
    let video_source = NativeVideoSource::new(VideoResolution {
        // The resolution is arbitrary and doesn't really matter, since livekit adapts to any resolution we capture
        width: 1920,
        height: 1080,
    });

    let track = local_participant
        .publish_track(
            LocalTrack::Video(LocalVideoTrack::create_video_track(
                "video",
                RtcVideoSource::Native(video_source.clone()),
            )),
            TrackPublishOptions {
                source: livekit::track::TrackSource::Camera,
                ..Default::default()
            },
        )
        .await
        .context("Failed to publish video track")?;

    Ok((track.sid(), video_source))
}

impl Drop for MediaPipeline {
    fn drop(&mut self) {
        self.publish_audio_mixer_task.abort();
        self.subscriber_audio_mixer_task.abort();

        self.ui_controller.stop();
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

struct CompositorObeliskBridgeSink {
    audio_tx: mpsc::Sender<Frame<RawAudio>>,
    video_tx: VideoSinks,
}

impl fmt::Debug for CompositorObeliskBridgeSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompositorObeliskBridgeSink").finish()
    }
}

impl opentalk_compositor::Sink for CompositorObeliskBridgeSink {
    fn on_audio_frame(&mut self, frame: Frame<RawAudio>) -> Result<()> {
        if let Err(e) = self.audio_tx.try_send(frame) {
            log::debug!("Failed to send audio sample from compositor to audiomixer, {e}");
        }

        Ok(())
    }

    fn on_video_frame(&mut self, buffer: Vec<u8>) -> Result<()> {
        let mut video_tx = self.video_tx.blocking_lock();

        for callback in video_tx.values_mut() {
            callback(&buffer);
        }

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

fn maybe_resize<'a>(
    resizer: &mut Resizer,
    src: &'a [u8],
    dst: &'a mut [u8],
    target_width: usize,
    target_height: usize,
) -> &'a [u8] {
    if target_width == WIDTH && target_height == HEIGHT {
        return src;
    }

    const COLOR: ezk_image::ColorInfo = ezk_image::ColorInfo::YUV(ezk_image::YuvColorInfo {
        transfer: ezk_image::ColorTransfer::Linear,
        primaries: ezk_image::ColorPrimaries::BT709,
        space: ezk_image::ColorSpace::BT709,
        full_range: false,
    });

    resizer
        .resize(
            Image::new(
                PixelFormat::I420,
                PixelFormatPlanes::infer_i420(src, WIDTH, HEIGHT),
                WIDTH,
                HEIGHT,
                COLOR,
                8,
            )
            .expect("src must always be a valid WIDTH * HEIGHT I420 image"),
            Image::new(
                PixelFormat::I420,
                PixelFormatPlanes::infer_i420(&mut *dst, target_width, target_height),
                target_width,
                target_height,
                COLOR,
                8,
            )
            .expect("dst must always be a valid target_width * target_height I420 image"),
        )
        .expect("both inputs are I420 images");

    dst
}

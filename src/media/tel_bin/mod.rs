// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::codec::{Codec, MediaType};
use super::track::create_track_src;
use super::util::ElementConvenienceExt;
use super::{MediaEvent, Target, TrackController};
use crate::sip::{SdpSendCodec, SrtpInfo};
use anyhow::{bail, Context, Result};
use gio::prelude::*;
use gst::{glib, prelude::*};
use gst_webrtc::WebRTCRTPTransceiverDirection;
use multimap::MultiMap;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Weak};
use tokio::sync::{mpsc, oneshot, Mutex, Notify};
use types::signaling::media::MediaSessionType;

mod rtpbin_wrapper;

/// Bin responsible for transmitting the compositor's A/V stream to the call-in peer while also receiving their media
/// and forwarding it to the conference using webrtcbin.
pub struct TelBin {
    pub(crate) bin: gst::Bin,

    pub(crate) rtp: rtpbin_wrapper::RtpBin,

    video_sink_tee: Option<gst::Element>,
    audio_sink_tee: gst::Element,

    publish_state: Arc<Mutex<ToPublishState>>,

    send_paths: HashMap<u32, SendPath>,
}

enum ReceivePath {
    Dtmf {
        dtmfdepay: gst::Element,
    },
    Audio {
        queue: gst::Element,
        depay: gst::Element,
        decode: gst::Element,
        audiomixer_pad: gst::Pad,
    },
    Video {
        queue: gst::Element,
        depay: gst::Element,
        decode: gst::Element,
        convert: gst::Element,
        input_selector_pad: gst::Pad,
    },
}

struct SendPath {
    codec: Codec,

    tee_pad: gst::Pad,
    queue1: gst::Element,
    convert: gst::Element,
    scale: gst::Element,
    encode: gst::Element,
    encode_caps: gst::Element,
    payload: gst::Element,
    queue2: gst::Element,
}

struct ToPublishState {
    receive_paths: MultiMap<u32, ReceivePath>,

    // All received audio is linked to this mixer
    audio_mixer_state: AudioMixerState,

    // All received video is linked to this input selector
    video_input_select_state: VideoInputSelectorState,

    webrtcbin: Option<gst::Element>,
}

#[derive(Clone)]
enum AudioMixerState {
    None,
    Valve {
        audio_mixer: gst::Element,
        valve: gst::Element,
    },
}

enum VideoInputSelectorState {
    None,
    Valve {
        input_selector: gst::Element,
        valve: gst::Element,
    },
}

#[derive(Debug)]
pub struct TelBinCompositorSink {
    bin: gst::Bin,
    rtp_bin: gst::Bin,

    audio_sink_pad: gst::GhostPad,
}

impl compositor::Sink for TelBinCompositorSink {
    fn on_play(&mut self) -> Result<()> {
        self.bin.set_state(gst::State::Playing)?;
        self.rtp_bin.set_state(gst::State::Playing)?;
        Ok(())
    }

    fn video(&self) -> Option<gst::GhostPad> {
        self.bin
            .sink_pads()
            .into_iter()
            .find(|pad| pad.name() == "video-sink")
            .and_then(|pad| pad.downcast().ok())
    }

    fn audio(&self) -> gst::GhostPad {
        self.audio_sink_pad.clone()
    }

    fn bin(&self) -> gst::Bin {
        self.bin.clone()
    }

    fn requires_eos(&self) -> bool {
        false
    }
}

impl TelBin {
    pub async fn new(
        ready_to_send: Arc<Notify>,
        with_video: bool,
    ) -> Result<(Self, TelBinCompositorSink, TrackController)> {
        let bin = gst::Bin::new();

        let rtp = rtpbin_wrapper::RtpBin::new(ready_to_send).context("Failed to create RtpBin")?;

        bin.add(rtp.bin())?;

        let state = Arc::new(Mutex::new(ToPublishState {
            receive_paths: MultiMap::new(),
            audio_mixer_state: AudioMixerState::None,
            video_input_select_state: VideoInputSelectorState::None,
            webrtcbin: None,
        }));

        let bin_weak = bin.downgrade();
        let weak_state = Arc::downgrade(&state);
        rtp.bin().connect_pad_added(move |_rtp_bin, pad| {
            if let Err(e) = rtp_on_pad_added(pad, &bin_weak, &weak_state) {
                log::error!("Failed to handle pad added to RtpBin, {e:?}");
            }
        });

        // Setup audio path
        //   +---------------------------------------------------
        //   |                                          +---------
        // ghostpad -> audiomixer -> audio-tee          | rtpbin
        //   |          /               \   \           |
        //   |      clocksync            \  encode -> ghostpad -> ...
        //   |        /                   \             |
        //   | tracksrc                     encode -> ghostpad -> .....
        //   |                                          +---------------
        //   +-----------------------------------------------------------
        let (track_controller, tracksrc) =
            create_track_src().context("Failed to create tracksrc")?;
        let clocksync = gst::ElementFactory::make("clocksync").build()?;

        let subscriber_audio_mixer = gst::ElementFactory::make("audiomixer")
            .property("ignore-inactive-pads", true)
            .property("force-live", true)
            .property("min-upstream-latency", 300_000_000u64)
            .property(
                "start-time-selection",
                gst_base::AggregatorStartTimeSelection::First,
            )
            .build()?;

        let audio_tee = gst::ElementFactory::make("tee")
            .name("audio-tee")
            .property("allow-not-linked", true)
            .build()?;

        bin.add_many([&tracksrc, &clocksync, &subscriber_audio_mixer, &audio_tee])?;

        subscriber_audio_mixer.link(&audio_tee)?;

        let audiomixer_sink_ghostpad = subscriber_audio_mixer
            .request_pad_simple("sink_%u")
            .context("Failed to request pad 'sink_%u' from audiomixer")?;
        let audiomixer_sink_tracksrc = subscriber_audio_mixer
            .request_pad_simple("sink_%u")
            .context("Failed to request pad 'sink_%u' from audiomixer")?;

        gst::Element::link_many([&tracksrc, &clocksync])?;
        clocksync
            .static_pad("src")
            .context("Failed to get static-pad 'src' of clocksync")?
            .link(&audiomixer_sink_tracksrc)?;

        let audio_pad = gst::GhostPad::builder_with_target(&audiomixer_sink_ghostpad)?
            .name("audio-sink")
            .build();
        bin.add_pad(&audio_pad)?;

        let rtp_bin = rtp.bin().clone();

        let mut this = Self {
            bin: bin.clone(),
            rtp,
            audio_sink_tee: audio_tee,
            video_sink_tee: None,
            publish_state: state,
            send_paths: HashMap::new(),
        };

        if with_video {
            this.enable_video(false).await?;
        }

        Ok((
            this,
            TelBinCompositorSink {
                bin,
                rtp_bin,
                audio_sink_pad: audio_pad,
            },
            track_controller,
        ))
    }

    pub async fn enable_video(&mut self, set_new_elements_to_playing: bool) -> Result<()> {
        if self.video_sink_tee.is_some() {
            return Ok(());
        }

        let video_tee = gst::ElementFactory::make("tee")
            .name("video-tee")
            .property("allow-not-linked", true)
            .build()?;
        let videoconvert = gst::ElementFactory::make("videoconvertscale").build()?;
        let videorate = gst::ElementFactory::make("videorate")
            .property("drop-only", true)
            .build()?;

        self.bin.add_many([&videoconvert, &videorate, &video_tee])?;
        gst::Element::link_many([&videoconvert, &videorate, &video_tee])?;

        let videoconvert_sink = videoconvert
            .static_pad("sink")
            .context("Failed to get static-pad 'sink' of videoconvert")?;

        let video_pad = gst::GhostPad::builder_with_target(&videoconvert_sink)?
            .name("video-sink")
            .build();
        self.bin.add_pad(&video_pad)?;

        if set_new_elements_to_playing {
            videoconvert.set_state_async(gst::State::Playing).await?;
            videorate.set_state_async(gst::State::Playing).await?;
            video_tee.set_state_async(gst::State::Playing).await?;
        }

        self.video_sink_tee = Some(video_tee);

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_sender(
        &mut self,
        session_id: u32,
        srtp_info: Option<&SrtpInfo>,
        codec_info: &SdpSendCodec,
        rtp_udpsink: gst::Element,
        rtcp_udpsrc: gst::Element,
        rtcp_udpsink: gst::Element,
        remote_rtp_addr: SocketAddr,
        remote_rtcp_addr: SocketAddr,
    ) -> Result<()> {
        self.rtp
            .set_pt_caps(codec_info.payload, codec_info.codec.caps());

        let pad = self
            .rtp
            .add_sender(
                session_id,
                srtp_info,
                rtp_udpsink,
                rtcp_udpsrc,
                rtcp_udpsink,
                remote_rtp_addr,
                remote_rtcp_addr,
            )
            .await
            .context("Failed to add sender to RtpBin")?;

        let (convert, scale) = match codec_info.codec.media_type() {
            MediaType::Audio => ("audioconvert", "audioresample"),
            MediaType::Video => ("videoconvert", "videoscale"),
        };

        let queue1 = gst::ElementFactory::make("queue").build()?;
        let convert = gst::ElementFactory::make(convert).build()?;
        let scale = gst::ElementFactory::make(scale).build()?;
        let capsfilter = gst::ElementFactory::make("capsfilter")
            .property("caps", codec_info.pre_encode_caps.clone())
            .build()?;

        let encode = codec_info
            .codec
            .encode_element(&codec_info.codec_params)
            .build()?;
        let encode_caps = gst::ElementFactory::make("capssetter")
            .property("caps", codec_info.post_encode_caps.clone())
            .build()?;
        let payload = codec_info
            .codec
            .payload_element()
            .property("pt", codec_info.payload)
            .build()?;
        let queue2 = gst::ElementFactory::make("queue").build()?;

        self.bin.add_many([
            &queue1,
            &convert,
            &scale,
            &capsfilter,
            &encode,
            &encode_caps,
            &payload,
            &queue2,
        ])?;

        gst::Element::link_many([
            &queue1,
            &convert,
            &scale,
            &capsfilter,
            &encode,
            &encode_caps,
            &payload,
            &queue2,
        ])?;

        let tee: &gst::Element = match codec_info.codec.media_type() {
            MediaType::Audio => &self.audio_sink_tee,
            MediaType::Video => self
                .video_sink_tee
                .as_ref()
                .context("Tried to add video stream without video support")?,
        };

        let tee_src_pad = tee
            .request_pad_simple("src_%")
            .context("Failed to request pad 'src_%' from tee")?;

        tee_src_pad.link(
            &queue1
                .static_pad("sink")
                .context("Failed to get static-pad 'sink' from queue")?,
        )?;

        queue2
            .static_pad("src")
            .context("Failed to get static-pad 'src' from queue")?
            .link(&pad)?;

        self.send_paths.insert(
            session_id,
            SendPath {
                codec: codec_info.codec,
                tee_pad: tee_src_pad,
                queue1: queue1.clone(),
                convert: convert.clone(),
                scale: scale.clone(),
                encode: encode.clone(),
                encode_caps: encode_caps.clone(),
                payload: payload.clone(),
                queue2: queue2.clone(),
            },
        );

        queue1
            .call_async_future(move |queue1| -> Result<()> {
                queue1.set_state(gst::State::Playing)?;
                convert.set_state(gst::State::Playing)?;
                scale.set_state(gst::State::Playing)?;
                capsfilter.set_state(gst::State::Playing)?;
                encode.set_state(gst::State::Playing)?;
                encode_caps.set_state(gst::State::Playing)?;
                payload.set_state(gst::State::Playing)?;
                queue2.set_state(gst::State::Playing)?;
                Ok(())
            })
            .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_receiver(
        &mut self,
        session_id: u32,
        srtp_info: Option<&SrtpInfo>,
        codec: Codec,
        pt: u32,
        rtp_udpsrc: gst::Element,
        rtcp_udpsrc: gst::Element,
        rtcp_udpsink: gst::Element,
        remote_rtcp_addr: SocketAddr,
    ) -> Result<()> {
        self.rtp.set_pt_caps(pt, codec.caps());

        self.publish_state
            .lock()
            .await
            .receive_paths
            .insert_many(session_id, []);

        self.rtp
            .add_receiver(
                session_id,
                srtp_info,
                rtp_udpsrc,
                rtcp_udpsrc,
                rtcp_udpsink,
                remote_rtcp_addr,
            )
            .await
    }

    pub async fn remove_sender(&mut self, session_id: u32) -> Result<()> {
        self.rtp
            .remove_sender(session_id)
            .await
            .context("rtp::remove_sender failed")?;

        let SendPath {
            codec,
            tee_pad,
            queue1,
            convert,
            scale,
            encode,
            encode_caps,
            payload,
            queue2,
        } = self
            .send_paths
            .remove(&session_id)
            .context("Failed to remove non existent send_path")?;

        match codec.media_type() {
            MediaType::Audio => self.audio_sink_tee.release_request_pad(&tee_pad),
            MediaType::Video => self
                .video_sink_tee
                .as_ref()
                .context("Tried to add video stream without video support")?
                .release_request_pad(&tee_pad),
        }

        gst::Element::unlink_many([
            &queue1,
            &convert,
            &scale,
            &encode,
            &encode_caps,
            &payload,
            &queue2,
        ]);

        self.bin
            .call_async_future(move |bin| -> Result<()> {
                queue1.set_state(gst::State::Null)?;
                convert.set_state(gst::State::Null)?;
                scale.set_state(gst::State::Null)?;
                encode.set_state(gst::State::Null)?;
                encode_caps.set_state(gst::State::Null)?;
                payload.set_state(gst::State::Null)?;
                queue2.set_state(gst::State::Null)?;

                bin.remove_many([
                    &queue1,
                    &convert,
                    &scale,
                    &encode,
                    &encode_caps,
                    &payload,
                    &queue2,
                ])?;

                Ok(())
            })
            .await
    }

    pub async fn remove_receiver(&mut self, session_id: u32) -> Result<()> {
        self.rtp.remove_receiver(session_id).await?;

        let mut state = self.publish_state.lock().await;
        // This might be a bit racy so no error here, just silent return
        let Some(receive_paths) = state.receive_paths.remove(&session_id) else {
            log::error!("No receive paths when removing receiver for session id {session_id}");
            return Ok(());
        };

        for receive_path in receive_paths {
            match receive_path {
                ReceivePath::Dtmf { dtmfdepay } => {
                    self.bin
                        .call_async_future(move |bin| -> Result<()> {
                            dtmfdepay.set_state(gst::State::Null)?;
                            bin.remove(&dtmfdepay)?;
                            Ok(())
                        })
                        .await?;
                }
                ReceivePath::Audio {
                    queue,
                    depay,
                    decode,
                    audiomixer_pad,
                } => {
                    let AudioMixerState::Valve {
                        ref audio_mixer, ..
                    } = state.audio_mixer_state
                    else {
                        bail!("No audiomixer set")
                    };

                    gst::Element::unlink_many([&queue, &depay, &decode]);
                    decode
                        .static_pad("src")
                        .context("Failed to get static-pad 'src' of decode")?
                        .unlink(&audiomixer_pad)?;
                    audio_mixer.release_request_pad(&audiomixer_pad);

                    self.bin
                        .call_async_future(move |bin| -> Result<()> {
                            queue.set_state(gst::State::Null)?;
                            depay.set_state(gst::State::Null)?;
                            decode.set_state(gst::State::Null)?;
                            bin.remove_many([&queue, &depay, &decode])?;
                            Ok(())
                        })
                        .await?;
                }
                ReceivePath::Video {
                    queue,
                    depay,
                    decode,
                    convert,
                    input_selector_pad,
                } => {
                    let VideoInputSelectorState::Valve {
                        ref input_selector, ..
                    } = state.video_input_select_state
                    else {
                        bail!("No audiomixer set")
                    };

                    gst::Element::unlink_many([&queue, &depay, &decode, &convert]);
                    convert
                        .static_pad("src")
                        .context("Failed to get static-pad 'src' from decode element")?
                        .unlink(&input_selector_pad)?;
                    input_selector.release_request_pad(&input_selector_pad);

                    self.bin
                        .call_async_future(move |bin| -> Result<()> {
                            queue.set_state(gst::State::Null)?;
                            depay.set_state(gst::State::Null)?;
                            decode.set_state(gst::State::Null)?;
                            convert.set_state(gst::State::Null)?;
                            bin.remove_many([&queue, &depay, &decode, &convert])?;
                            Ok(())
                        })
                        .await?;
                }
            }
        }

        Ok(())
    }

    pub async fn create_publish(
        &mut self,
        events_tx: mpsc::UnboundedSender<MediaEvent>,
    ) -> Result<String> {
        let mut state = self.publish_state.lock().await;

        if state.webrtcbin.is_some() {
            bail!("Tried to create publish but was already created");
        }

        let AudioMixerState::Valve {
            audio_mixer: _,
            valve: audio_mixer_valve,
        } = state.audio_mixer_state.clone()
        else {
            bail!("Tried to create publish but there's no audio stream");
        };

        let mut publish_bin = r#"
            webrtcbin name=webrtc-publish bundle-policy=max-bundle

            volume name=audio-input !
                audioconvert !
                audioresample !
                level audio-level-meta=true !
                audio/x-raw,rate=48000,channels=2,layout=interleaved !
                opusenc !
                rtpopuspay pt=97 auto-header-extension=true !
                application/x-rtp,media=audio,encoding-name=OPUS,payload=97,extmap-1=(string)<"", urn:ietf:params:rtp-hdrext:ssrc-audio-level, "vad=on"> !
                queue !
                webrtc-publish.
        "#.to_owned();

        if matches!(
            state.video_input_select_state,
            VideoInputSelectorState::Valve { .. }
        ) {
            publish_bin += r#"
                videoconvertscale name=video-input !
                    queue !
                    vp8enc deadline=1 !
                    rtpvp8pay pt=98 !
                    queue !
                    webrtc-publish.
            "#;
        }

        let publish_bin =
            gst::parse::bin_from_description_with_name(&publish_bin, false, "publish")
                .context("failed to parse webrtc-publish bin")?;

        self.bin.add(&publish_bin)?;

        let webrtc = publish_bin
            .by_name("webrtc-publish")
            .context("webrtc defined in pipeline")?;

        state.webrtcbin = Some(webrtc.clone());

        // Connect audio path
        let audio_input = publish_bin
            .by_name("audio-input")
            .context("No audio-input found")?;
        let audio_input_sink = audio_input
            .static_pad("sink")
            .context("Element without sink pad")?;

        let webrtc_publish_audio_sink = gst::GhostPad::builder_with_target(&audio_input_sink)?
            .name("audio-sink")
            .build();

        publish_bin.add_pad(&webrtc_publish_audio_sink)?;

        audio_mixer_valve.link_pads(Some("src"), &publish_bin, Some("audio-sink"))?;
        audio_mixer_valve.set_property("drop", false);

        // Set audio transceiver direction to sendonly
        let transceiver = webrtc
            .emit_by_name::<Option<gst_webrtc::WebRTCRTPTransceiver>>("get-transceiver", &[&0i32])
            .context("no transceiver in 0")?;

        transceiver.set_direction(WebRTCRTPTransceiverDirection::Sendonly);
        transceiver.set_property("do-nack", false);

        // Connect video path, if exists
        if let VideoInputSelectorState::Valve { valve, .. } = &state.video_input_select_state {
            // Set transceiver direction to sendonly
            let transceiver = webrtc
                .emit_by_name::<Option<gst_webrtc::WebRTCRTPTransceiver>>(
                    "get-transceiver",
                    &[&1i32],
                )
                .context("no transceiver in 0")?;

            transceiver.set_direction(WebRTCRTPTransceiverDirection::Sendonly);
            transceiver.set_property("do-nack", false);

            let video_input = publish_bin
                .by_name("video-input")
                .context("No audio-input found")?;
            let video_input_sink = video_input
                .static_pad("sink")
                .context("Element without sink pad")?;

            let webrtc_publish_video_sink = gst::GhostPad::builder_with_target(&video_input_sink)?
                .name("video-sink")
                .build();

            publish_bin.add_pad(&webrtc_publish_video_sink)?;

            valve
                .static_pad("src")
                .context("Failed to get static-pad 'src' from valve")?
                .link(&webrtc_publish_video_sink)?;

            valve.set_property("drop", false);
        }

        let events_tx_ = events_tx.clone();
        webrtc.connect("on-ice-candidate", true, move |values| {
            let mline_index = values[1].get::<u32>().expect("mline_index is guint");
            let candidate = values[2].get::<String>().expect("candidate is gchararray");

            events_tx_
                .send(MediaEvent::IceCandidate {
                    // TODO: When adding screen share support this must be changed
                    target: Target::Publish(MediaSessionType::Video),
                    candidate,
                    mline_index,
                })
                .ok();

            None
        });

        webrtc.connect("notify::ice-gathering-state", true, move |values| {
            let webrtcbin = values[0]
                .get::<gst::Element>()
                .expect("values[0] of notify::ice-gathering-state is a gst::Element");

            let state =
                webrtcbin.property::<gst_webrtc::WebRTCICEGatheringState>("ice-gathering-state");

            if state == gst_webrtc::WebRTCICEGatheringState::Complete {
                // ICE candidate gathering is complete - send signal to task
                if events_tx
                    // TODO: When adding screen share support this must be changed
                    .send(MediaEvent::NoMoreIceCandidates(Target::Publish(
                        MediaSessionType::Video,
                    )))
                    .is_err()
                {
                    log::error!("Failed to send end-of-candidates signal to task");
                }
            }

            None
        });

        // Wait for webrtcbin to be ready
        let notify = Arc::new(Notify::new());
        let notify_ = notify.clone();

        webrtc.connect("on-negotiation-needed", true, move |_| {
            notify_.notify_one();
            None
        });

        publish_bin
            .call_async_future(|publish_bin| publish_bin.set_state(gst::State::Playing))
            .await?;

        notify.notified().await;

        // Create SDP offer
        let (tx, rx) = oneshot::channel();
        let webrtc_weak = webrtc.downgrade();
        let on_create_offer = gst::Promise::with_change_func(move |offer| {
            let Some(webrtc) = webrtc_weak.upgrade() else {
                return;
            };

            let offer = offer.ok().flatten().and_then(|offer| {
                offer
                    .get::<gst_webrtc::WebRTCSessionDescription>("offer")
                    .ok()
            });

            let offer = if let Some(offer) = offer {
                offer
            } else {
                log::error!("create-offer failed");
                return;
            };

            log::debug!("setting local description to\n{}", offer.sdp());

            webrtc.emit_by_name::<()>("set-local-description", &[&offer, &None::<gst::Promise>]);

            if tx.send(offer.sdp().to_string()).is_err() {
                log::error!("Failed to forward sdp offer back to task");
            }
        });

        webrtc.emit_by_name::<()>("create-offer", &[&None::<gst::Structure>, &on_create_offer]);

        rx.await.context("Failed to create SDP offer")
    }

    pub async fn on_publish_response(&mut self, response: String) -> Result<()> {
        let state = self.publish_state.lock().await;

        let Some(webrtc) = &state.webrtcbin else {
            bail!("Tried to give SDP response to non existent publisher")
        };

        let response = gst_webrtc::WebRTCSessionDescription::new(
            gst_webrtc::WebRTCSDPType::Answer,
            gst_sdp::SDPMessage::parse_buffer(response.as_bytes())?,
        );

        log::debug!("setting remote description to\n{}", response.sdp());

        webrtc.emit_by_name::<()>(
            "set-remote-description",
            &[&response, &None::<gst::Promise>],
        );

        Ok(())
    }

    pub async fn on_publish_sdp_candidate(
        &mut self,
        mline: u32,
        candidate: Option<String>,
    ) -> Result<()> {
        let state = self.publish_state.lock().await;

        let webrtcbin = state
            .webrtcbin
            .as_ref()
            .context("No webrtc publish session created yet")?;

        webrtcbin.emit_by_name::<()>("add-ice-candidate", &[&mline, &candidate]);

        Ok(())
    }
}

fn rtp_on_pad_added(
    pad: &gst::Pad,
    bin_weak: &glib::WeakRef<gst::Bin>,
    weak_state: &Weak<Mutex<ToPublishState>>,
) -> Result<()> {
    if pad.direction() != gst::PadDirection::Src {
        return Ok(());
    }
    let Some(bin) = bin_weak.upgrade() else {
        return Ok(());
    };
    let Some(state) = weak_state.upgrade() else {
        return Ok(());
    };

    let session_id = rtpbin_wrapper::session_id_of_rtp_src_pad(&pad.name())
        .context("Failed to get session_id from src_pad")?;

    let caps = {
        let pad = pad
            .downcast_ref::<gst::GhostPad>()
            .context("Added pad isn't a GhostPad")?;
        let target = pad.target().context("Added GhostPad has no target")?;
        target
            .current_caps()
            .context("Added GhostPad has no caps")?
    };

    let caps = caps.structure(0).context("pad caps has no entries")?;
    let encoding_name = caps
        .get::<String>("encoding-name")
        .context("pad caps has no 'encoding-name' set")?;

    log::debug!("Receiving {encoding_name}");

    let mut state = state.blocking_lock();

    // Handle DTMF caps
    if encoding_name == "TELEPHONE-EVENT" {
        let dtmfdepay = gst::ElementFactory::make("opentalk-rtpdtmfdepay").build()?;
        bin.add(&dtmfdepay)?;
        pad.link(
            &dtmfdepay
                .static_pad("sink")
                .context("Failed to get static-pad 'sink' in dtmfdepay ")?,
        )?;
        dtmfdepay.set_state(gst::State::Playing)?;

        state
            .receive_paths
            .insert(session_id, ReceivePath::Dtmf { dtmfdepay });

        return Ok(());
    }

    let codec = Codec::from_encoding_name(&encoding_name)
        .with_context(|| format!("Unknown encoding-name {encoding_name}"))?;
    let queue = gst::ElementFactory::make("queue").build()?;
    let depay = gst::ElementFactory::make(codec.depayload_element()).build()?;
    let decode = gst::ElementFactory::make(codec.decode_element()).build()?;

    bin.add_many([&queue, &depay, &decode])?;
    gst::Element::link_many([&queue, &depay, &decode])?;
    pad.link(
        &queue
            .static_pad("sink")
            .context("Failed to get static-pad 'sink' of queue")?,
    )?;

    match codec.media_type() {
        MediaType::Audio => {
            let audio_mixer = match &state.audio_mixer_state {
                AudioMixerState::None => {
                    let publish_audio_mixer = gst::ElementFactory::make("audiomixer")
                        .property("min-upstream-latency", 100_000_000u64)
                        .property("ignore-inactive-pads", true)
                        .property("force-live", true)
                        .property(
                            "start-time-selection",
                            gst_base::AggregatorStartTimeSelection::First,
                        )
                        .build()?;
                    let publish_audio_mixer_valve = gst::ElementFactory::make("valve")
                        .property("drop", true)
                        .build()?;

                    bin.add_many([&publish_audio_mixer, &publish_audio_mixer_valve])?;
                    gst::Element::link_many([&publish_audio_mixer, &publish_audio_mixer_valve])?;

                    publish_audio_mixer.set_state(gst::State::Playing)?;
                    publish_audio_mixer_valve.set_state(gst::State::Playing)?;

                    state.audio_mixer_state = AudioMixerState::Valve {
                        audio_mixer: publish_audio_mixer.clone(),
                        valve: publish_audio_mixer_valve,
                    };

                    publish_audio_mixer
                }
                AudioMixerState::Valve { audio_mixer, .. } => audio_mixer.clone(),
            };

            let decode_src = decode
                .static_pad("src")
                .context("Failed to get static-pad 'src' from decode element")?;

            let audiomixer_pad = audio_mixer
                .request_pad_simple("sink_%u")
                .context("Failed to request pad 'sink_%u' in audiomixer")?;
            decode_src.link(&audiomixer_pad)?;

            queue.set_state(gst::State::Playing)?;
            depay.set_state(gst::State::Playing)?;
            decode.set_state(gst::State::Playing)?;

            state.receive_paths.insert(
                session_id,
                ReceivePath::Audio {
                    queue,
                    depay,
                    decode,
                    audiomixer_pad,
                },
            );
        }
        MediaType::Video => {
            let input_selector = match &state.video_input_select_state {
                VideoInputSelectorState::None => {
                    let videotestsrc = gst::ElementFactory::make("videotestsrc")
                        .property("is-live", true)
                        .property_from_str("pattern", "black")
                        .build()
                        .context("Failed to build videotestsrc")?;

                    let input_selector = gst::ElementFactory::make("input-selector")
                        .property("sync-streams", false)
                        .property_from_str("sync-mode", "clock")
                        .build()
                        .context("Failed to build input-selector for video streams")?;

                    let input_selector_valve = gst::ElementFactory::make("valve")
                        .property("drop", true)
                        .build()
                        .context("Failed to build valve for video streams")?;

                    bin.add_many([&videotestsrc, &input_selector, &input_selector_valve])?;

                    videotestsrc
                        .static_pad("src")
                        .context("Failed to get static-pad 'src' from videotestsrc")?
                        .link(
                            &input_selector
                                .request_pad_simple("sink_%u")
                                .context("Failed to link videotestsrc to input-selector")?,
                        )?;

                    input_selector.link(&input_selector_valve)?;

                    videotestsrc.set_state(gst::State::Playing)?;
                    input_selector.set_state(gst::State::Playing)?;
                    input_selector_valve.set_state(gst::State::Playing)?;

                    state.video_input_select_state = VideoInputSelectorState::Valve {
                        input_selector: input_selector.clone(),
                        valve: input_selector_valve,
                    };

                    input_selector
                }
                VideoInputSelectorState::Valve { input_selector, .. } => input_selector.clone(),
            };

            let convert = gst::ElementFactory::make("videoconvertscale").build()?;

            bin.add(&convert)?;
            decode.link(&convert)?;
            let convert_src = convert
                .static_pad("src")
                .context("Failed to get static-pad 'src' from convert element")?;

            let input_selector_pad = input_selector
                .request_pad_simple("sink_%u")
                .context("Failed to request 'sink_%u' pad from input-selector")?;

            convert_src.link(&input_selector_pad)?;

            queue.set_state(gst::State::Playing)?;
            depay.set_state(gst::State::Playing)?;
            decode.set_state(gst::State::Playing)?;
            convert.set_state(gst::State::Playing)?;

            // Lazy: Always set the newest video stream as the active one
            input_selector.set_property("active-pad", &input_selector_pad);

            state.receive_paths.insert(
                session_id,
                ReceivePath::Video {
                    queue,
                    depay,
                    decode,
                    convert,
                    input_selector_pad,
                },
            );
        }
    }

    Ok(())
}

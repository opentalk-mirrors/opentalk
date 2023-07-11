// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::websocket::TrickleCandidate;
use anyhow::{bail, Context, Result};
use bytes::Bytes;
use glib::source::Continue;
use gst::glib;
use gst::prelude::*;
use gst_webrtc::WebRTCRTPTransceiverDirection;
use sip_bin::SipBin;
use std::net::IpAddr;
use std::sync::Arc;
use std::thread;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

mod handle;
pub mod port_pool;
mod sip_bin;
pub mod track;
mod webrtc_subscribe;

pub use handle::{HandleError, HandleEvent, MediaTaskHandle, Target};
pub use track::{Track, TrackController};

enum MediaTaskMsg {
    CreateSipSession {
        offer: Bytes,
        answer_tx: oneshot::Sender<(TrackController, String)>,
    },
    SipReInvite {
        offer: Bytes,
        answer_tx: oneshot::Sender<String>,
    },
    CreatePublish,
    CreateSubscribe {
        uuid: Uuid,
        offer: String,
    },
    RemoveSubscribe {
        uuid: Uuid,
    },
    ReceivePublishResponse {
        response: String,
    },
    PublishMute {
        mute: bool,
    },
    CandidateReceived {
        candidate: TrickleCandidate,
        target: Target,
    },
    EndOfCandidatesReceived {
        target: Target,
    },
}

struct MediaTask {
    pub_addr: IpAddr,
    task_tx: mpsc::UnboundedSender<HandleEvent>,
    pipeline: gst::Pipeline,
    sip_bin: Option<SipBin>,
}

impl Drop for MediaTask {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}

impl MediaTask {
    fn new(pub_addr: IpAddr, task_tx: mpsc::UnboundedSender<HandleEvent>) -> Self {
        let pipeline = gst::Pipeline::new(None);

        Self {
            pub_addr,
            task_tx,
            pipeline,
            sip_bin: None,
        }
    }

    fn run(mut self, mut rx: mpsc::UnboundedReceiver<MediaTaskMsg>) {
        while let Some(cmd) = rx.blocking_recv() {
            match cmd {
                MediaTaskMsg::CreateSipSession { offer, answer_tx } => {
                    if let Err(e) = self.on_sip_sdp_offer(offer, answer_tx) {
                        log::error!("failed to handle sdp offer, {:?}", e);
                        self.task_tx.send(HandleEvent::Error).ok();
                        break;
                    }
                }
                MediaTaskMsg::SipReInvite { offer, answer_tx } => {
                    if let Err(e) = self.on_reinvite(offer, answer_tx) {
                        log::error!("failed to handle reinvite, {:?}", e);
                        self.task_tx.send(HandleEvent::Error).ok();
                        break;
                    }
                }
                MediaTaskMsg::CandidateReceived { candidate, target } => {
                    if let Err(e) = self.on_sdp_candidate(candidate, target) {
                        log::error!("failed to handle sdp candidate, {:?}", e);
                        self.task_tx.send(HandleEvent::Error).ok();
                        break;
                    }
                }
                MediaTaskMsg::EndOfCandidatesReceived { target } => {
                    if let Err(e) = self.on_sdp_end_of_candidates(target) {
                        log::error!("failed to handle end of candidates, {:?}", e);
                        self.task_tx.send(HandleEvent::Error).ok();
                        break;
                    }
                }
                MediaTaskMsg::CreatePublish => {
                    if let Err(e) = self.create_publish() {
                        log::error!("failed to create publish, {:?}", e);
                        self.task_tx.send(HandleEvent::Error).ok();
                        break;
                    }
                }
                MediaTaskMsg::CreateSubscribe { uuid, offer } => {
                    if let Err(e) = self.create_subscribe(uuid, offer) {
                        log::error!("failed to create subscribe {}, {:?}", uuid, e);
                        self.task_tx.send(HandleEvent::Error).ok();
                        break;
                    }
                }
                MediaTaskMsg::RemoveSubscribe { uuid } => {
                    if let Err(e) = self.remove_subscribe(uuid) {
                        log::error!("failed to remove subscribe {}, {:?}", uuid, e);
                    }
                }
                MediaTaskMsg::ReceivePublishResponse { response } => {
                    if let Err(e) = self.on_publish_response(response) {
                        log::error!("failed to handle publish response, {:?}", e);
                        self.task_tx.send(HandleEvent::Error).ok();
                        break;
                    }
                }
                MediaTaskMsg::PublishMute { mute } => {
                    if let Err(e) = self.on_publish_mute(mute) {
                        log::error!("failed to mute, {:?}", e);
                        self.task_tx.send(HandleEvent::Error).ok();
                        break;
                    }
                }
            }
        }
    }

    fn on_sip_sdp_offer(
        &mut self,
        offer: Bytes,
        answer_tx: oneshot::Sender<(TrackController, String)>,
    ) -> Result<()> {
        if self.sip_bin.is_some() {
            bail!("Already created a sip_bin");
        }

        let (sip_bin, answer) = SipBin::new(self.pub_addr, offer)?;
        self.pipeline.add(&sip_bin.bin)?;

        let mixer = gst::ElementFactory::make("audiomixer")
            .name("subscriber-audiomix")
            .build()?;
        self.pipeline.add(&mixer)?;

        mixer
            .link_pads(Some("src"), &sip_bin.bin, Some("sip-audio-sink"))
            .context("failed to link mixer to sip-bin")?;

        let (controller, tracksrc) = track::create_track_src()?;

        self.pipeline.add(&tracksrc)?;
        tracksrc.link(&mixer)?;

        // Watch the pipeline bus
        let bus = self.pipeline.bus().context("failed to get pipeline bus")?;

        let task_tx = self.task_tx.clone();

        self.sip_bin = Some(sip_bin);

        if answer_tx.send((controller, answer)).is_err() {
            bail!("failed to send SDP response to handle")
        }

        let pipeline_weak = self.pipeline.downgrade();
        bus.add_watch(move |_, msg| {
            let mut do_continue = true;

            match msg.view() {
                gst::MessageView::Error(error) => {
                    if let Some(source) = error.src() {
                        if source.name() == "rtp-watchdog" {
                            if task_tx.send(HandleEvent::Error).is_err() {
                                log::error!("Unable to send rtp timeout error to signaling");
                            }

                            return Continue(false);
                        }
                    }

                    log::error!("Unhandled pipeline error, {:?}", error);
                }
                gst::MessageView::Latency(_) => {
                    if let Some(pipeline) = pipeline_weak.upgrade() {
                        if let Err(e) = pipeline.recalculate_latency() {
                            log::error!("Failed to recalculate latency, {e}");
                        }
                    }
                }
                gst::MessageView::Element(element) => {
                    if let Some(structure) = element.structure() {
                        if structure.name() == "dtmf-event" {
                            if let Ok(digit) = structure.get::<i32>("number") {
                                do_continue = task_tx.send(HandleEvent::DtmfDigit(digit)).is_ok();
                            }
                        }
                    }
                }
                _ => (),
            }

            Continue(do_continue)
        })?;

        drop(bus);

        self.pipeline.set_state(gst::State::Playing)?;

        Ok(())
    }

    fn on_sdp_candidate(&mut self, candidate: TrickleCandidate, target: Target) -> Result<()> {
        self.send_sdp_candidate(target, &candidate.sdp_m_line_index, &candidate.candidate)
    }

    fn on_sdp_end_of_candidates(&mut self, target: Target) -> Result<()> {
        self.send_sdp_candidate(target, &0u32, &None::<String>)
    }

    fn send_sdp_candidate(
        &mut self,
        target: Target,
        media_line_index: &u32,
        candidate: &dyn ToValue,
    ) -> Result<()> {
        let target_name = match target {
            Target::Publish => "publish".to_string(),
            Target::Subscribe(uuid) => format!("subscribe:{}", uuid),
        };

        let subscribe_bin = match self.pipeline.by_name(&target_name) {
            Some(subscribe_bin) => subscribe_bin,
            None => {
                log::warn!("tried to add-ice-candidate for {target_name}");
                return Ok(());
            }
        };

        let subscribe_bin = subscribe_bin
            .downcast::<gst::Bin>()
            .expect("target bin is a gst::Bin");
        let webrtc_subscribe = subscribe_bin
            .by_name(&format!("webrtc-{target_name}"))
            .context(format!("failed to get webrtc from webrtc-{target_name}"))?;
        webrtc_subscribe.emit_by_name::<()>("add-ice-candidate", &[media_line_index, candidate]);

        Ok(())
    }

    fn on_reinvite(&mut self, offer: Bytes, answer_tx: oneshot::Sender<String>) -> Result<()> {
        if let Some(sip_bin) = &mut self.sip_bin {
            match sip_bin.update(offer) {
                Ok(answer) => {
                    answer_tx.send(answer).ok();
                }
                Err(e) => log::error!("Failed to handle re-invite, {:?}", e),
            }

            Ok(())
        } else {
            bail!("received SipReInvite event without a sip_bin initialized");
        }
    }

    fn create_publish(&mut self) -> Result<()> {
        let publish_bin = r#"
            webrtcbin name=webrtc-publish bundle-policy=max-bundle

            volume name=audio-input !
                audioconvert !
                audioresample !
                queue !
                level audio-level-meta=true !
                audio/x-raw,rate=48000,channels=2,layout=interleaved !
                opusenc !
                rtpopuspay pt=97 auto-header-extension=true !
                application/x-rtp,media=audio,encoding-name=OPUS,payload=97,extmap-1=(string)<"", urn:ietf:params:rtp-hdrext:ssrc-audio-level, "vad=on"> !
                queue !
                webrtc-publish.
        "#;

        let publish_bin = gst::parse_bin_from_description_with_name(publish_bin, false, "publish")
            .context("failed to parse webrtc-publish bin")?;

        self.pipeline.add(&publish_bin)?;

        let webrtc = publish_bin
            .by_name("webrtc-publish")
            .context("webrtc defined in pipeline")?;

        let audio_input = publish_bin
            .by_name("audio-input")
            .context("No audio-input found")?;

        let audio_input_sink = audio_input
            .static_pad("sink")
            .context("Element without sink pad")?;

        let webrtc_publish_audio_sink =
            gst::GhostPad::with_target(Some("webrtc-publish-sink"), &audio_input_sink)?;

        publish_bin.add_pad(&webrtc_publish_audio_sink)?;

        let sip_bin = self
            .pipeline
            .by_name("sip-bin")
            .context("no sip-bin in pipeline")?;

        sip_bin.link_pads(
            Some("sip-audio-src"),
            &publish_bin,
            Some("webrtc-publish-sink"),
        )?;

        // Set transceiver direction to sendonly
        let transceiver = webrtc
            .emit_by_name::<Option<gst_webrtc::WebRTCRTPTransceiver>>("get-transceiver", &[&0])
            .context("no transceiver in 0")?;

        transceiver.set_direction(WebRTCRTPTransceiverDirection::Sendonly);
        transceiver.set_property("do-nack", false);

        self.connect_ice_candidate_gathering_hooks(&webrtc, Target::Publish);

        let task_tx = self.task_tx.clone();
        webrtc.connect("on-negotiation-needed", true, move |values| {
            let webrtc = values[0]
                .get::<gst::Element>()
                .expect("values[0] of on-negotiation-needed is a gst::Element");

            let task_tx = task_tx.clone();

            let webrtc_weak = webrtc.downgrade();
            let on_create_offer = gst::Promise::with_change_func(move |offer| {
                let Some(webrtc) = webrtc_weak.upgrade() else { return };

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

                webrtc
                    .emit_by_name::<()>("set-local-description", &[&offer, &None::<gst::Promise>]);

                if task_tx
                    .send(HandleEvent::PublishOffer(offer.sdp().to_string()))
                    .is_err()
                {
                    log::error!("bug: failed to send sdp offer");
                }
            });

            webrtc.emit_by_name::<()>("create-offer", &[&None::<gst::Structure>, &on_create_offer]);

            None
        });

        publish_bin.set_state(gst::State::Playing)?;

        Ok(())
    }

    fn on_publish_response(&mut self, response: String) -> Result<()> {
        let webrtc = self
            .pipeline
            .by_name("webrtc-publish")
            .context("webrtc-publish must be in pipeline")?;

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

    fn create_subscribe(&mut self, uuid: Uuid, offer: String) -> Result<()> {
        let subscribe_bin = format!(
            r#"
            webrtcbin name=webrtc-subscribe:{uuid} bundle-policy=max-bundle
            "#,
            uuid = uuid
        );

        let subscribe_bin = gst::parse_bin_from_description_with_name(
            &subscribe_bin,
            false,
            &format!("subscribe:{}", uuid),
        )
        .context("failed to parse webrtc-subscribe bin")?;

        self.pipeline.add(&subscribe_bin)?;

        let webrtc = subscribe_bin
            .by_name(&format!("webrtc-subscribe:{}", uuid))
            .context("webrtc defined in pipeline")?;

        self.connect_ice_candidate_gathering_hooks(&webrtc, Target::Subscribe(uuid));

        // Get mixer
        let mixer = self
            .pipeline
            .by_name("subscriber-audiomix")
            .context("missing mixer")?;

        // Handle incoming media streams
        webrtc.connect(
            "pad-added",
            true,
            webrtc_subscribe::webrtcbin_on_pad_added(
                uuid,
                subscribe_bin.downgrade(),
                mixer.downgrade(),
            ),
        );

        // Negotiate SDP when the webrtcbin object is ready
        webrtc.connect(
            "on-negotiation-needed",
            true,
            webrtc_subscribe::webrtcbin_on_negotation_needed(uuid, self.task_tx.clone(), offer),
        );

        subscribe_bin.set_state(gst::State::Playing)?;

        Ok(())
    }

    fn remove_subscribe(&mut self, uuid: Uuid) -> Result<()> {
        let subscribe_bin =
            if let Some(subscribe_bin) = self.pipeline.by_name(&format!("subscribe:{}", uuid)) {
                subscribe_bin
                    .downcast::<gst::Bin>()
                    .expect("subscribe bin is a gst::Bin")
            } else {
                log::warn!("tried to remove nonexistent subscribe:{}", uuid);
                return Ok(());
            };

        log::debug!("Removing subscriber {uuid}");

        let mixer = self
            .pipeline
            .by_name("subscriber-audiomix")
            .context("no audiomixer in pipeline")?;

        // Clean up every pad
        for pad in subscribe_bin.pads() {
            assert_eq!(pad.direction(), gst::PadDirection::Src);

            // Get the peer of the source pad, which is the sink request-pad into the subscriber-audiomix
            let mixer_sink_pad = pad.peer().context("subscribe bin pad")?;

            pad.unlink(&mixer_sink_pad)
                .context("Failed to unlink subscribe_bin's pad from its own peer")?;

            // Release the request pad on the mixer, we don't need it anymore
            mixer.release_request_pad(&mixer_sink_pad);
        }

        // Remove the subscribe bin from the pipeline
        self.pipeline.remove(&subscribe_bin)?;

        // Set subscribe_bin to Null, destroying it
        subscribe_bin.set_state(gst::State::Null)?;

        Ok(())
    }

    fn connect_ice_candidate_gathering_hooks(&self, webrtcbin: &gst::Element, target: Target) {
        let task_tx = self.task_tx.clone();
        webrtcbin.connect("on-ice-candidate", true, move |values| {
            let mline_index = values[1].get::<u32>().expect("mline_index is guint");
            let candidate = values[2].get::<String>().expect("candidate is gchararray");

            task_tx
                .send(HandleEvent::IceCandidate {
                    target,
                    candidate,
                    mline_index,
                })
                .ok();

            None
        });

        let task_tx = self.task_tx.clone();
        webrtcbin.connect("notify::ice-gathering-state", true, move |values| {
            let webrtcbin = values[0]
                .get::<gst::Element>()
                .expect("values[0] of notify::ice-gathering-state is a gst::Element");

            let state =
                webrtcbin.property::<gst_webrtc::WebRTCICEGatheringState>("ice-gathering-state");

            if state == gst_webrtc::WebRTCICEGatheringState::Complete {
                let _ = task_tx.send(HandleEvent::NoMoreIceCandidates(target));
            }

            None
        });
    }

    fn on_publish_mute(&mut self, mute: bool) -> Result<()> {
        if let Some(volume) = self.pipeline.by_name("audio-input") {
            volume.set_property("mute", mute);
        }

        Ok(())
    }
}

pub fn spawn_task(pub_addr: IpAddr, call_id: &str) -> MediaTaskHandle {
    let (handle_tx, task_rx) = mpsc::unbounded_channel();
    let (task_tx, handle_rx) = mpsc::unbounded_channel();

    let handle_tx = Arc::new(handle_tx);

    thread::Builder::new()
        .name(format!("media-task-{call_id}"))
        .spawn(move || MediaTask::new(pub_addr, task_tx).run(task_rx))
        .expect("Failed to spawn media-task thread");

    MediaTaskHandle {
        tx: handle_tx,
        rx: handle_rx,
    }
}

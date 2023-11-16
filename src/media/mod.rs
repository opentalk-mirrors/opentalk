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
use tokio::sync::{mpsc, oneshot, Notify};
use uuid::Uuid;

pub mod port_pool;
mod sip_bin;
pub mod track;
mod webrtc_subscribe;

pub use sip_bin::init_custom_rtpdtmfdepay;
pub use track::{Track, TrackController};

/// Webrtc session of an event
#[derive(Clone, Copy)]
pub enum Target {
    Publish,
    Subscribe(Uuid),
}

/// Asynchronous events emitted by the media pipeline
pub enum MediaEvent {
    Error,
    DtmfDigit(i32),
    IceCandidate {
        target: Target,
        candidate: String,
        mline_index: u32,
    },
    NoMoreIceCandidates(Target),
}

pub struct MediaPipeline {
    events_tx: mpsc::UnboundedSender<MediaEvent>,
    events_rx: mpsc::UnboundedReceiver<MediaEvent>,
    pipeline: gst::Pipeline,
    sip_bin: SipBin,
}

impl Drop for MediaPipeline {
    fn drop(&mut self) {
        self.pipeline.call_async(|pipeline| {
            let _ = pipeline.set_state(gst::State::Null);
        });
    }
}

impl MediaPipeline {
    /// # Returns
    ///
    /// - Self
    /// - track controller to play different audio tracks
    /// - string containing the SIP SDP Answer
    pub async fn new(
        pub_addr: IpAddr,
        sip_sdp_offer: Bytes,
        ready_to_send: Arc<Notify>,
    ) -> Result<(Self, TrackController, String)> {
        let (events_tx, events_rx) = mpsc::unbounded_channel();

        let pipeline = gst::Pipeline::new(None);

        // Create the SIP-BIN, it has a single sink for audio and zero or more audio sources
        let (sip_bin, answer) = SipBin::new(pub_addr, sip_sdp_offer, ready_to_send)?;
        pipeline.add(&sip_bin.bin)?;

        // Create a audiomixer to mix all webrtc-subscriptions into a single audio stream, which can be sent to the SIP
        // peer
        let subscriber_mixer = gst::ElementFactory::make("audiomixer")
            .name("subscriber-audiomix")
            .property("ignore-inactive-pads", true)
            .build()?;
        pipeline.add(&subscriber_mixer)?;

        subscriber_mixer
            .link_pads(Some("src"), &sip_bin.bin, Some("sip-audio-sink"))
            .context("failed to link mixer to sip-bin")?;

        // Create a track-source and link it to the subscriber-audiomix, so the SIP peer will hear the tracks played
        let (controller, tracksrc) = track::create_track_src()?;

        pipeline.add(&tracksrc)?;
        tracksrc.link(&subscriber_mixer)?;

        // Create a second audiomixer to combine all audio streams received by us from the SIP peer, to be later
        // sent via webrtc.
        // As long as there is no publishing webrtc connection the audio stream is dumped into a fakesink.
        let publisher_mixer = gst::ElementFactory::make("audiomixer")
            .name("publisher-audiomix")
            .property("ignore-inactive-pads", true)
            .build()?;
        let fakesink = gst::ElementFactory::make("fakesink")
            .name("publisher-fakesink")
            .build()?;
        pipeline.add_many(&[&publisher_mixer, &fakesink])?;
        publisher_mixer.link(&fakesink)?;

        // Handle pads added to sip-bin
        let publisher_mixer_weak = publisher_mixer.downgrade();
        sip_bin.bin.connect_pad_added(move |_, pad| {
            // Connect the pad directly to the audiomixer
            let Some(mixer) = publisher_mixer_weak.upgrade() else {
                return;
            };

            let res = (|| {
                let sink_pad = mixer
                    .request_pad_simple("sink_%u")
                    .context("Failed to request pad from audiomixer")?;
                pad.link(&sink_pad).context("Failed to link")
            })();

            if let Err(e) = res {
                log::error!("Failed to handle new pad in sip-bin, {e:?}");
            }
        });

        // TODO: Handle pads removed by the sip-bin (tough edge case)
        // Currently pad from the sip-bin are directly linked to the publisher-audiomix's requested pads. Not handling
        // these removed pads leads to these request-pads accumulating, though this should not be a huge problem since
        // it shouldn't happen very often and the audiomixer has `ignore-inactive-pads` set to true.

        // Watch the pipeline bus
        let bus = pipeline.bus().context("failed to get pipeline bus")?;

        let events_tx_ = events_tx.clone();

        let pipeline_weak = pipeline.downgrade();
        bus.add_watch(move |_, msg| {
            let mut do_continue = true;

            match msg.view() {
                gst::MessageView::Error(error) => {
                    if let Some(source) = error.src() {
                        if source.name().contains("rtp-watchdog") {
                            log::error!(
                                "Detected Problems with sending or receiving RTP ({} triggered)",
                                source.name()
                            );

                            if events_tx_.send(MediaEvent::Error).is_err() {
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
                                do_continue = events_tx_.send(MediaEvent::DtmfDigit(digit)).is_ok();
                            }
                        }
                    }
                }
                _ => (),
            }

            Continue(do_continue)
        })?;

        drop(bus);

        pipeline
            .call_async_future(|pipeline| pipeline.set_state(gst::State::Playing))
            .await?;

        let this = Self {
            events_rx,
            events_tx,
            pipeline,
            sip_bin,
        };

        Ok((this, controller, answer))
    }

    pub async fn wait_for_event(&mut self) -> MediaEvent {
        self.events_rx
            .recv()
            .await
            .expect("self also holds the sender side of the channel, so this should never fail")
    }

    pub fn on_sdp_candidate(&mut self, candidate: TrickleCandidate, target: Target) -> Result<()> {
        self.send_sdp_candidate(target, &candidate.sdp_m_line_index, &candidate.candidate)
    }

    pub fn on_sdp_end_of_candidates(&mut self, target: Target) -> Result<()> {
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

    pub fn on_reinvite(&mut self, offer: Bytes) -> Result<String> {
        self.sip_bin
            .update(offer)
            .context("Failed to handle re-invite")
    }

    pub async fn create_publish(&mut self) -> Result<String> {
        if self.pipeline.by_name("publish").is_some() {
            bail!("publish bin has already been created once");
        }

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
            gst::GhostPad::with_target(Some("audio-sink"), &audio_input_sink)?;

        publish_bin.add_pad(&webrtc_publish_audio_sink)?;

        let mixer = self
            .pipeline
            .by_name("publisher-audiomix")
            .context("Failed to get publisher-audiomix, has the sip-bin been created?")?;
        let publisher_fakesink = self
            .pipeline
            .by_name("publisher-fakesink")
            .context("Failed to get publisher-fakesink")?;

        mixer.unlink(&publisher_fakesink);
        self.pipeline.remove(&publisher_fakesink)?;
        publisher_fakesink.set_state(gst::State::Null)?;

        mixer.link_pads(Some("src"), &publish_bin, Some("audio-sink"))?;

        // Set transceiver direction to sendonly
        let transceiver = webrtc
            .emit_by_name::<Option<gst_webrtc::WebRTCRTPTransceiver>>("get-transceiver", &[&0i32])
            .context("no transceiver in 0")?;

        transceiver.set_direction(WebRTCRTPTransceiverDirection::Sendonly);
        transceiver.set_property("do-nack", false);

        self.connect_ice_candidate_gathering_hooks(&webrtc, Target::Publish);

        publish_bin
            .call_async_future(|publish_bin| publish_bin.set_state(gst::State::Playing))
            .await?;

        // Wait for webrtcbin to be ready
        let notify = Arc::new(Notify::new());
        let notify_clone = notify.clone();

        let notified = notify.notified();

        webrtc.connect("on-negotiation-needed", true, move |_| {
            notify_clone.notify_waiters();
            None
        });

        notified.await;

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

    pub fn on_publish_response(&mut self, response: String) -> Result<()> {
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

    pub async fn create_subscribe(&mut self, uuid: Uuid, offer: String) -> Result<String> {
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

        subscribe_bin
            .call_async_future(|subscribe_bin| subscribe_bin.set_state(gst::State::Playing))
            .await?;

        // Parse the SDP text into a gstreamer WebRTCSessionDescription
        let offer = gst_sdp::SDPMessage::parse_buffer(offer.as_bytes())
            .context("Failed to parse SDP offer to gst::SDPMessage")?;
        let offer =
            gst_webrtc::WebRTCSessionDescription::new(gst_webrtc::WebRTCSDPType::Offer, offer);

        // Set the offer as "remote" description to respond with a local description
        log::debug!("settings remote description to \n{}", offer.sdp());
        webrtc.emit_by_name::<()>("set-remote-description", &[&offer, &None::<gst::Promise>]);

        // Create callback to receive the SDP response from the webrtcbin
        let (tx, rx) = oneshot::channel();
        let webrtc_weak = webrtc.downgrade();
        let promise = gst::Promise::with_change_func(move |answer| {
            let Some(webrtc) = webrtc_weak.upgrade() else {
                return;
            };

            // Extract the SDP Answer from the many layers of wrapper types
            let Ok(Some(structure)) = answer else {
                log::error!("Failed to create SDP Answer, {:?}", answer);
                return;
            };
            let Ok(answer) = structure.get::<gst_webrtc::WebRTCSessionDescription>("answer") else {
                return;
            };

            // Set our SDP response as local SDP
            log::debug!("setting local description to\n{}", answer.sdp());
            webrtc.emit_by_name::<()>("set-local-description", &[&answer, &None::<gst::Promise>]);

            // Send the SDP answer back to the signaling task
            if tx.send(answer.sdp().to_string()).is_err() {
                log::error!("Failed to send SDP answer back to task");
            }
        });

        webrtc.emit_by_name::<()>("create-answer", &[&None::<gst::Structure>, &promise]);

        rx.await.context("Failed to create SDP answer")
    }

    pub async fn remove_subscribe(&mut self, uuid: Uuid) -> Result<()> {
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
        subscribe_bin
            .call_async_future(|subscribe_bin| subscribe_bin.set_state(gst::State::Null))
            .await?;

        Ok(())
    }

    fn connect_ice_candidate_gathering_hooks(&self, webrtcbin: &gst::Element, target: Target) {
        let events_tx = self.events_tx.clone();
        webrtcbin.connect("on-ice-candidate", true, move |values| {
            let mline_index = values[1].get::<u32>().expect("mline_index is guint");
            let candidate = values[2].get::<String>().expect("candidate is gchararray");

            events_tx
                .send(MediaEvent::IceCandidate {
                    target,
                    candidate,
                    mline_index,
                })
                .ok();

            None
        });

        let events_tx = self.events_tx.clone();
        webrtcbin.connect("notify::ice-gathering-state", true, move |values| {
            let webrtcbin = values[0]
                .get::<gst::Element>()
                .expect("values[0] of notify::ice-gathering-state is a gst::Element");

            let state =
                webrtcbin.property::<gst_webrtc::WebRTCICEGatheringState>("ice-gathering-state");

            if state == gst_webrtc::WebRTCICEGatheringState::Complete {
                // ICE candidate gathering is complete - send signal to task
                if events_tx
                    .send(MediaEvent::NoMoreIceCandidates(target))
                    .is_err()
                {
                    log::error!("Failed to send end-of-candidates signal to task");
                }
            }

            None
        });
    }

    pub fn on_publish_mute(&mut self, mute: bool) -> Result<()> {
        if let Some(volume) = self.pipeline.by_name("audio-input") {
            volume.set_property("mute", mute);
        }

        Ok(())
    }
}

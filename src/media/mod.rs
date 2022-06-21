use anyhow::{bail, Context, Result};
use bytes::Bytes;
use gio::prelude::*;
use gst::glib;
use gst::prelude::*;
use gst_webrtc::WebRTCRTPTransceiverDirection;
use sip_bin::SipBin;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

mod handle;
pub mod port_pool;
mod sip_bin;
pub mod track;

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
        let pipeline = gst::Pipeline::new(Some("nordstream"));

        Self {
            pub_addr,
            task_tx,
            pipeline,
            sip_bin: None,
        }
    }

    async fn run(mut self, mut rx: mpsc::UnboundedReceiver<MediaTaskMsg>) {
        while let Some(cmd) = rx.recv().await {
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

        let mixer = gst::ElementFactory::make("audiomixer", Some("subscriber-audiomix"))?;
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

        bus.add_watch_local(move |_, msg| {
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
                gst::MessageView::Element(element) => {
                    if let Some(structure) = element.structure() {
                        if structure.name() == "dtmf-event" {
                            if let Ok(digit) = structure.get::<i32>("number") {
                                do_continue = task_tx.send(HandleEvent::DtmfDigit(digit)).is_ok();
                            }
                        }
                    }
                }
                gst::MessageView::PropertyNotify(property) => {
                    let (obj, name, value) = property.get();

                    if name == "ice-gathering-state" {
                        let obj_name = obj.name();

                        let target = if obj_name == "webrtc-publish" {
                            Some(Target::Publish)
                        } else if obj_name.starts_with("webrtc-subscribe") {
                            if let Some((_, uuid)) = obj_name.split_once(':') {
                                if let Ok(uuid) = Uuid::from_str(uuid) {
                                    Some(Target::Subscribe(uuid))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        if let Some(target) = target {
                            if let Some(value) = value.and_then(|value| {
                                value.get::<gst_webrtc::WebRTCICEGatheringState>().ok()
                            }) {
                                if value == gst_webrtc::WebRTCICEGatheringState::Complete {
                                    do_continue = task_tx
                                        .send(HandleEvent::NoMoreIceCandidates(target))
                                        .is_ok();
                                }
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
            webrtcbin name=webrtc-publish

            volume name=audio-input !
                audioresample !
                level audio-level-meta=true !
                audioconvert !
                audio/x-raw,rate=48000,channels=2,layout=interleaved !
                opusenc !
                rtpopuspay pt=97 auto-header-extension=true !
                application/x-rtp,media=audio,encoding-name=OPUS,payload=97,extmap-1=(string)<"", urn:ietf:params:rtp-hdrext:ssrc-audio-level, "vad=on"> !
                queue name=opus-output
        "#;

        let publish_bin = gst::parse_bin_from_description_with_name(publish_bin, false, "publish")
            .context("failed to parse webrtc-publish bin")?;

        self.pipeline.set_state(gst::State::Paused)?;

        self.pipeline.add(&publish_bin)?;

        let webrtc = publish_bin
            .by_name("webrtc-publish")
            .context("webrtc defined in pipeline")?;

        let webrtc_sink = webrtc
            .request_pad_simple("sink_%u")
            .context("Unable to request webrtcbin sink pad")?;

        let audio_input = publish_bin
            .by_name("audio-input")
            .context("No audio-input found")?;

        let audio_input_sink = audio_input
            .static_pad("sink")
            .context("Element without sink pad")?;

        let opus_output = publish_bin
            .by_name("opus-output")
            .context("No opus-output found")?;

        let opus_output_src = opus_output
            .static_pad("src")
            .context("Element without src pad")?;

        opus_output_src.link(&webrtc_sink)?;

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
            .emit_by_name("get-transceiver", &[&0])?
            .context("no transceiver in 0")?
            .get::<gst_webrtc::WebRTCRTPTransceiver>()?;

        transceiver.set_direction(WebRTCRTPTransceiverDirection::Sendonly);
        transceiver.set_property("do-nack", &false)?;

        let task_tx = self.task_tx.clone();

        webrtc.connect("on-ice-candidate", true, move |values| {
            let mline_index = values[1].get::<u32>().expect("mline_index is guint");
            let candidate = values[2].get::<String>().expect("candidate is gchararray");

            task_tx
                .send(HandleEvent::IceCandidate {
                    target: Target::Publish,
                    candidate,
                    mline_index,
                })
                .ok();

            None
        })?;

        webrtc.add_property_notify_watch(None, true);

        let webrtc_clone = webrtc.clone();
        let task_tx = self.task_tx.clone();
        webrtc.connect("on-negotiation-needed", true, move |_| {
            let webrtc = webrtc_clone.clone();

            let task_tx = task_tx.clone();

            let on_create_offer = gst::Promise::with_change_func(move |offer| {
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

                if let Err(e) =
                    webrtc.emit_by_name("set-local-description", &[&offer, &None::<gst::Promise>])
                {
                    log::error!("failed to set local description, {}", e);
                }

                if task_tx
                    .send(HandleEvent::PublishOffer(offer.sdp().to_string()))
                    .is_err()
                {
                    log::error!("bug: failed to send sdp offer");
                }
            });

            if let Err(e) = webrtc_clone
                .emit_by_name("create-offer", &[&None::<gst::Structure>, &on_create_offer])
            {
                log::error!("on-negotiation-needed-cb: failed to create offer, {}", e);
            }

            None
        })?;

        self.pipeline.set_state(gst::State::Playing)?;

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

        webrtc.emit_by_name(
            "set-remote-description",
            &[&response, &None::<gst::Promise>],
        )?;

        Ok(())
    }

    fn create_subscribe(&mut self, uuid: Uuid, offer: String) -> Result<()> {
        let subscribe_bin = format!(
            r#"
            webrtcbin name=webrtc-subscribe:{uuid} !
            
            rtpopusdepay name=opus-input !
                opusdec !
                audio/x-raw,clock-rate=48000,channels=2,layout=interleaved !
                queue name=audio-output 
            "#,
            uuid = uuid
        );

        let subscribe_bin = gst::parse_bin_from_description_with_name(
            &subscribe_bin,
            false,
            &format!("subscribe:{}", uuid),
        )
        .context("failed to parse webrtc-subscribe bin")?;

        self.pipeline.set_state(gst::State::Paused)?;

        self.pipeline.add(&subscribe_bin)?;

        let webrtc = subscribe_bin
            .by_name(&format!("webrtc-subscribe:{}", uuid))
            .context("webrtc defined in pipeline")?;

        // ==== LINK audio-output -> mixer.sink_%u

        let audio_output = subscribe_bin
            .by_name("audio-output")
            .context("No audio-input found")?;

        let audio_output_src_pad = audio_output
            .static_pad("src")
            .context("failed to get src sink from audio-output")?;

        let audio_output_ghost_pad =
            gst::GhostPad::with_target(Some("subscribe-audio-output"), &audio_output_src_pad)?;
        subscribe_bin.add_pad(&audio_output_ghost_pad)?;

        // Get mixer
        let mixer = self
            .pipeline
            .by_name("subscriber-audiomix")
            .context("missing mixer")?;

        let mixer_sink = mixer
            .request_pad_simple("sink_%u")
            .context("failed to request mixer sink pad")?;

        audio_output_ghost_pad
            .link(&mixer_sink)
            .context("failed to link audio-output src to mixer sink")?;

        let offer = gst_sdp::SDPMessage::parse_buffer(offer.as_bytes())?;
        let offer =
            gst_webrtc::WebRTCSessionDescription::new(gst_webrtc::WebRTCSDPType::Offer, offer);

        self.pipeline.set_state(gst::State::Playing)?;

        log::debug!("settings remote description to \n{}", offer.sdp());

        webrtc.emit_by_name("set-remote-description", &[&offer, &None::<gst::Promise>])?;

        let webrtc_clone = webrtc.clone();
        let task_tx = self.task_tx.clone();
        let promise = gst::Promise::with_change_func(move |answer| {
            let answer = answer.ok().flatten().and_then(|answer| {
                answer
                    .get::<gst_webrtc::WebRTCSessionDescription>("answer")
                    .ok()
            });

            let answer = if let Some(answer) = answer {
                answer
            } else {
                log::error!("create-answer failed");
                return;
            };

            log::debug!("setting local description to\n{}", answer.sdp());

            if let Err(e) =
                webrtc.emit_by_name("set-local-description", &[&answer, &None::<gst::Promise>])
            {
                log::error!("failed to set local description, {}", e);
            }

            if task_tx
                .send(HandleEvent::SubscribeAnswer {
                    uuid,
                    answer: answer.sdp().to_string(),
                })
                .is_err()
            {
                log::error!("bug: failed to send sdp offer");
            }
        });

        if let Err(e) =
            webrtc_clone.emit_by_name("create-answer", &[&None::<gst::Structure>, &promise])
        {
            log::error!("failed to create answer, {}", e);
        }

        Ok(())
    }

    fn remove_subscribe(&mut self, uuid: Uuid) -> Result<()> {
        let subscribe_bin =
            if let Some(subscribe_bin) = self.pipeline.by_name(&format!("subscribe:{}", uuid)) {
                subscribe_bin
            } else {
                log::warn!("tried to remove nonexistent subscribe:{}", uuid);
                return Ok(());
            };

        let mixer = self
            .pipeline
            .by_name("subscriber-audiomix")
            .context("no audiomixer in pipeline")?;

        self.pipeline.set_state(gst::State::Paused)?;

        let res = || -> Result<()> {
            // Unlink subscribe_bin from mixer and remove it frm the pipeline
            subscribe_bin.unlink(&mixer);
            self.pipeline.remove(&subscribe_bin)?;

            // Set subscribe_bin to Ready, then Null - destroying it
            subscribe_bin.set_state(gst::State::Ready)?;
            subscribe_bin.set_state(gst::State::Null)?;
            Ok(())
        }();

        self.pipeline.set_state(gst::State::Playing)?;

        // Check result of subscribe-bin removal
        res?;

        Ok(())
    }

    fn on_publish_mute(&mut self, mute: bool) -> Result<()> {
        if let Some(volume) = self.pipeline.by_name("audio-input") {
            volume.set_property("mute", mute)?;
        }

        Ok(())
    }
}

pub fn spawn_task(pub_addr: IpAddr, main_ctx: &glib::MainContext) -> MediaTaskHandle {
    let (handle_tx, task_rx) = mpsc::unbounded_channel();
    let (task_tx, handle_rx) = mpsc::unbounded_channel();

    let handle_tx = Arc::new(handle_tx);
    main_ctx.spawn(MediaTask::new(pub_addr, task_tx).run(task_rx));

    MediaTaskHandle {
        tx: handle_tx,
        rx: handle_rx,
    }
}

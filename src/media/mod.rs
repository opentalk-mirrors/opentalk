// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result};
use compositor::{MediaDescriptor, Mixer, Size, Speaker, WebRtcSource, WebRtcSourceParams};
use glib::source::Continue;
use gst::glib;
use gst::prelude::*;
use std::sync::Arc;
use tel_bin::TelBin;
use tokio::sync::{mpsc, Notify};
use types::signaling::media::{MediaSessionState, MediaSessionType, TrickleCandidate};

pub mod codec;
mod custom_rtpdtmfdepay;
pub mod port_pool;
mod tel_bin;
pub mod track;
mod util;

pub use custom_rtpdtmfdepay::init_custom_rtpdtmfdepay;
pub use track::{Track, TrackController};

/// Webrtc session of an event
#[derive(Clone, Copy)]
pub enum Target {
    Publish(MediaSessionType),
    Subscribe(MediaDescriptor),
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
    pub tel_bin: TelBin,
    pub with_video: bool,
    pub compositor: Mixer<WebRtcSource>,

    events_tx: mpsc::UnboundedSender<MediaEvent>,
    events_rx: mpsc::UnboundedReceiver<MediaEvent>,
}

impl MediaPipeline {
    /// # Returns
    ///
    /// - Self
    /// - track controller to play different audio tracks
    pub async fn new(
        ready_to_send: Arc<Notify>,
        with_video: bool,
    ) -> Result<(Self, TrackController)> {
        let (events_tx, events_rx) = mpsc::unbounded_channel();

        let mut compositor = Mixer::create(None, Size::HD, Speaker::default(), 6, with_video)
            .context("Failed to create compositor")?;

        let (tel_bin, sink, track_controller) = TelBin::new(ready_to_send, with_video).await?;

        compositor
            .link_sink("tel", sink)
            .context("Failed to link_sink")?;

        // Ouch, get the sink pipeline by getting the bin's parent
        // This pipeline is managed by the compositor
        let pipeline = tel_bin
            .bin
            .parent()
            .context("sink's bin hast no parent after being added to compositor")?
            .downcast::<gst::Pipeline>()
            .ok()
            .context("sink's parent is not a pipeline")?;

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
                    if let Some(mut src) = msg
                        .src()
                        .and_then(|obj| obj.downcast_ref::<gst::Element>())
                        .and_then(|element| element.parent())
                        .and_then(|element_parent_obj| {
                            element_parent_obj.downcast::<gst::Bin>().ok()
                        })
                    {
                        if src.recalculate_latency().is_err() {
                            log::error!("Failed latency recalculation of {}", src.name());
                        }

                        while let Some(parent) =
                            src.parent().and_then(|obj| obj.downcast::<gst::Bin>().ok())
                        {
                            if parent.recalculate_latency().is_err() {
                                log::error!("Failed latency recalculation of {}", parent.name());
                            }

                            src = parent;
                        }
                    }

                    if let Some(pipeline) = pipeline_weak.upgrade() {
                        if pipeline.recalculate_latency().is_err() {
                            log::error!("Failed latency recalculation of pipeline");
                        }
                    }
                }
                gst::MessageView::Element(element) => {
                    if let Some(structure) = element.structure() {
                        if structure.name() == "dtmf-event" {
                            if let Ok(digit) = structure.get::<i32>("number") {
                                log::trace!("Got DTMF digit: {digit}");

                                do_continue = events_tx_.send(MediaEvent::DtmfDigit(digit)).is_ok();
                            }
                        }
                    }
                }
                _ => (),
            }

            Continue(do_continue)
        })?;

        let this = Self {
            tel_bin,
            with_video,
            events_rx,
            events_tx,
            compositor,
        };

        Ok((this, track_controller))
    }

    pub async fn wait_for_event(&mut self) -> MediaEvent {
        self.events_rx
            .recv()
            .await
            .expect("self also holds the sender side of the channel, so this should never fail")
    }

    pub async fn enable_video(&mut self) -> Result<()> {
        self.tel_bin.enable_video(true).await?;
        self.compositor
            .enable_video()
            .context("Failed to enable video in compositor")?;

        Ok(())
    }

    pub async fn on_sdp_candidate(
        &mut self,
        candidate: TrickleCandidate,
        target: Target,
    ) -> Result<()> {
        match target {
            Target::Publish(media_session_type) => {
                assert_eq!(media_session_type, MediaSessionType::Video);
                //FIXME: fix in types and remove cast
                self.tel_bin
                    .on_publish_sdp_candidate(
                        candidate.sdp_m_line_index as u32,
                        Some(candidate.candidate),
                    )
                    .await
            }
            Target::Subscribe(desc) => {
                let subscribe = self
                    .compositor
                    .get_source(desc)
                    .context("Invalid StreamId for ICE candidate")?;

                subscribe
                    .receive_candidate(candidate.sdp_m_line_index as u32, &candidate.candidate);

                Ok(())
            }
        }
    }

    pub async fn on_sdp_end_of_candidates(&mut self, target: Target) -> Result<()> {
        match target {
            Target::Publish(media_session_type) => {
                assert_eq!(media_session_type, MediaSessionType::Video);
                self.tel_bin.on_publish_sdp_candidate(0, None).await
            }
            Target::Subscribe(desc) => {
                let subscribe = self
                    .compositor
                    .get_source(desc)
                    .context("Invalid StreamId for ICE candidate")?;

                subscribe.receive_end_of_candidates(0);

                Ok(())
            }
        }
    }

    pub async fn create_publish(&mut self) -> Result<String> {
        self.tel_bin.create_publish(self.events_tx.clone()).await
    }

    pub async fn on_publish_response(&mut self, response: String) -> Result<()> {
        self.tel_bin.on_publish_response(response).await
    }

    pub async fn create_subscribe(
        &mut self,
        desc: MediaDescriptor,
        display_name: &str,
        offer: String,
        has_audio: bool,
        has_video: bool,
    ) -> Result<String> {
        let events_tx = self.events_tx.clone();
        let params =
            WebRtcSourceParams::new(true).on_ice_candidate(move |mline_index, candidate| {
                if let Some(candidate) = candidate {
                    events_tx
                        .send(MediaEvent::IceCandidate {
                            target: Target::Subscribe(desc),
                            candidate,
                            mline_index,
                        })
                        .ok();
                } else {
                    events_tx
                        .send(MediaEvent::NoMoreIceCandidates(Target::Subscribe(desc)))
                        .ok();
                }
            });

        self.compositor
            .add_stream(
                desc,
                display_name.into(),
                params,
                MediaSessionState {
                    audio: has_audio,
                    video: has_video,
                },
            )
            .context("Failed to add stream")?;

        self.compositor
            .set_speaker(desc.participant_id)
            .context("Failed to set_speaker")?;

        let webrtc = self
            .compositor
            .get_source(desc)
            .expect("get_source must return some because the source was just added");

        let answer = webrtc.receive_offer(offer).await?;

        Ok(answer)
    }

    pub fn update_subscribe(
        &mut self,
        desc: MediaDescriptor,
        has_audio: bool,
        has_video: bool,
    ) -> Result<()> {
        self.compositor.set_status(
            desc,
            MediaSessionState {
                audio: has_audio,
                video: has_video,
            },
        )
    }

    pub async fn remove_subscribe(&mut self, desc: MediaDescriptor) -> Result<()> {
        self.compositor.remove_stream(desc)
    }
}

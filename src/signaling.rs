// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::http::{HttpClient, InvalidCredentials};
use crate::media::{MediaEvent, MediaPipeline, Target, Track, TrackController};
use crate::settings::Settings;
use crate::sip::{handle_reinvite, SessionState};
use crate::websocket::{JoinState, Participant, Websocket, WebsocketEvent};
use anyhow::{Context, Result};
use bytesstr::BytesStr;
use compositor::MediaDescriptor;
use sip_ua::invite::session::{Event, Session};
use std::fmt::Write;
use std::future::pending;
use std::sync::Arc;
use tokio::sync::broadcast;
use types::signaling::media::{MediaSessionState, MediaSessionType};

/// State of the signaling task
enum State {
    /// Running until state changes
    Running,

    /// An error occurred and the SIP session has to be terminated
    Quitting,

    /// The SIP session has ended (either via BYE or error)
    Terminated,
}

/// Per Call main-loop
///
/// Handles all the signaling associated with SIP (after accepting the Call)
/// and the websocket connection to the controller.
pub struct Signaling {
    settings: Arc<Settings>,

    http_client: Arc<HttpClient>,

    /// Name of the user (usually the phone number or Anonymous)
    name: Option<BytesStr>,

    /// The SIP session
    sip: Session,
    /// The websocket connection to the controller
    /// Will be initialized once the user typed in
    /// the correct digits to enter a room
    websocket: Option<Websocket>,

    /// Handle to the media task
    media: MediaPipeline,
    media_session: SessionState,

    /// Track controller to play audio tracks on demand to the SIP user
    track_controller: TrackController,
    publish_complete_pending: bool,

    /// Current state of the signaling
    state: State,

    /// DTMF ID is set before connecting the websocket
    dtmf_id: String,

    /// DTMF ID is set before connecting the websocket
    dtmf_pw: String,

    /// Is the client muted
    muted: bool,

    /// has the client the hand raised
    hand_raised: bool,

    /// List of participants inside the room,
    /// unused before initiating the websocket connection
    participants: Vec<Participant>,

    /// Application shutdown signal
    shutdown: broadcast::Receiver<()>,
}

impl Signaling {
    /// Create a new Signaling from a newly established SIP session
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        settings: Arc<Settings>,
        http_client: Arc<HttpClient>,
        name: Option<BytesStr>,
        sip: Session,
        media: MediaPipeline,
        media_session: SessionState,
        track_controller: TrackController,
        shutdown: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            settings,
            http_client,
            name,
            sip,
            websocket: None,
            media,
            media_session,
            track_controller,
            publish_complete_pending: true,
            state: State::Running,
            dtmf_id: String::new(),
            dtmf_pw: String::new(),
            muted: false,
            hand_raised: false,
            participants: Vec::new(),
            shutdown,
        }
    }

    /// Run until completion
    pub async fn run(&mut self) -> Result<()> {
        let (welcome_finished_channel_tx, mut welcome_finished_channel_rx) =
            broadcast::channel::<()>(3);
        let (closed_finished_channel_tx, mut closed_finished_channel_rx) =
            broadcast::channel::<()>(3);

        while matches!(self.state, State::Running) {
            tokio::select! {
                event = self.sip.drive() => {
                    match event? {
                        Event::RefreshNeeded(event) => event.process_default().await?,
                        Event::ReInviteReceived(event) => {
                            if let Err(e) = handle_reinvite(&mut self.media, &mut self.media_session, event).await {
                                log::error!("failed to handle re-invite, {:?}", e);
                                self.state = State::Quitting;
                            }
                        },
                        Event::Bye(event) => {
                            event.process_default().await?;
                            self.state = State::Terminated;
                        },
                        Event::Terminated => self.state = State::Terminated,
                    }
                }
                event = self.media.wait_for_event() => {
                    if let Err(e) = self.handle_media_event(event, welcome_finished_channel_tx.clone()).await {
                        log::error!("failed to handle media event, {:?}", e);
                        self.state = State::Quitting;
                    }
                }
                event = websocket_receive(&mut self.websocket) => {
                    if let Err(e) = self.handle_websocket_event(event, closed_finished_channel_tx.clone()).await {
                        log::error!("failed to handle websocket event, {:?}", e);
                        self.state = State::Quitting;
                    }
                }
                _ = welcome_finished_channel_rx.recv() => {
                    if let Err(e) = self.begin_publish_and_subscriptions().await {
                        log::error!("failed to create webrtc connections, {:?}", e);
                        self.state = State::Quitting;
                    }
                }
                _ = closed_finished_channel_rx.recv() => {
                    self.state = State::Quitting;
                }
                _ = self.shutdown.recv() => {
                    self.state = State::Quitting;
                }
            }
        }

        if matches!(self.state, State::Quitting) {
            self.sip.terminate().await?;
        }

        self.close_websocket().await?;

        Ok(())
    }

    async fn begin_publish_and_subscriptions(&mut self) -> Result<()> {
        let websocket = self.websocket.as_mut().context("No websocket connection")?;

        // Send publish command to media task
        let offer = self
            .media
            .create_publish()
            .await
            .context("Failed to create publishing webrtc connection")?;
        websocket
            .publish(&offer, MediaSessionType::Video)
            .await
            .context("Failed to send SDP publish offer via websocket")?;

        // Subscribe to all participants that currently publish
        for participant in &self.participants {
            if participant.control.left_at.is_none() && participant.media.video.is_some() {
                websocket
                    .subscribe(
                        MediaDescriptor {
                            participant_id: participant.id,
                            media_type: MediaSessionType::Video,
                        },
                        self.media.with_video,
                    )
                    .await?;
            }
        }

        Ok(())
    }

    async fn close_websocket(&mut self) -> Result<()> {
        if let Some(websocket) = &mut self.websocket {
            websocket.close().await?;
            self.websocket = None;
        }
        Ok(())
    }

    async fn handle_media_event(
        &mut self,
        event: MediaEvent,
        welcome_finished_responder: broadcast::Sender<()>,
    ) -> Result<()> {
        match event {
            MediaEvent::DtmfDigit(digit) if self.dtmf_pw.len() < 10 => {
                // Dialing into the session
                match digit {
                    11 => {
                        // #
                        self.dtmf_id.clear();
                        self.dtmf_pw.clear();
                    }
                    10 => {
                        // *
                    }
                    0..=9 => {
                        if self.dtmf_id.len() < 10 {
                            let _ = write!(&mut self.dtmf_id, "{}", digit);

                            if self.dtmf_id.len() == 10 {
                                self.track_controller.play_track(Track::WelcomePasscode);
                            }
                        } else {
                            let _ = write!(&mut self.dtmf_pw, "{}", digit);

                            if self.dtmf_pw.len() == 10 {
                                match self.join().await {
                                    Ok(_) => {
                                        self.track_controller.play_track_and_respond(
                                            Track::WelcomeUsage,
                                            welcome_finished_responder,
                                        );
                                    }
                                    Err(e) if e.is::<InvalidCredentials>() => {
                                        self.dtmf_id.clear();
                                        self.dtmf_pw.clear();

                                        self.track_controller.play_track(Track::InputInvalid);
                                        return Ok(());
                                    }
                                    Err(e) => {
                                        log::error!("failed to join, {}", e);
                                        return Err(e);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            MediaEvent::DtmfDigit(digit) => {
                if digit == 0 {
                    self.track_controller.play_track(Track::Silence)
                } else if digit == 1 {
                    if let Some(websocket) = &mut self.websocket {
                        self.muted = !self.muted;
                        websocket
                            .send_mute(
                                self.muted,
                                self.muted && (!self.media.with_video),
                                MediaSessionType::Video,
                            )
                            .await?;
                        self.track_controller.play_track(if self.muted {
                            Track::Muted
                        } else {
                            Track::Unmuted
                        });
                    }
                } else if digit == 2 {
                    if let Some(websocket) = &mut self.websocket {
                        self.hand_raised = !self.hand_raised;
                        self.track_controller.play_track(if self.hand_raised {
                            Track::HandRaised
                        } else {
                            Track::HandLowered
                        });
                        websocket.send_hand_action(self.hand_raised).await?;
                    }
                }
            }
            MediaEvent::Error => {
                self.state = State::Quitting;
            }
            MediaEvent::IceCandidate {
                target,
                candidate,
                mline_index,
            } => {
                if let Some(websocket) = &mut self.websocket {
                    match target {
                        Target::Publish(media_session_type) => {
                            websocket
                                .send_ice_candidate(
                                    None,
                                    media_session_type,
                                    candidate,
                                    mline_index,
                                )
                                .await?
                        }
                        Target::Subscribe(desc) => {
                            websocket
                                .send_ice_candidate(
                                    Some(desc.participant_id),
                                    desc.media_type,
                                    candidate,
                                    mline_index,
                                )
                                .await?
                        }
                    }
                }
            }
            MediaEvent::NoMoreIceCandidates(target) => {
                if let Some(websocket) = &mut self.websocket {
                    match target {
                        Target::Publish(media_session_type) => {
                            websocket
                                .publish_end_of_candidates(media_session_type)
                                .await?
                        }
                        Target::Subscribe(uuid) => {
                            websocket.subscribe_end_of_candidates(uuid).await?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_websocket_event(
        &mut self,
        event: Result<WebsocketEvent>,
        closed_finished_responder: broadcast::Sender<()>,
    ) -> Result<()> {
        let websocket = self.websocket.as_mut().expect("websocket must be set");

        match event? {
            WebsocketEvent::SubscribeOffer { offer, desc } => {
                let Some(participant) = self
                    .participants
                    .iter()
                    .find(|p| p.id == desc.participant_id)
                else {
                    log::debug!("Got SDP offer for subscription to unknown participant id={}. Might've just left after subscribing to it", desc.participant_id);
                    return Ok(());
                };

                if let Some(status) = &participant.media.video {
                    let answer = self
                        .media
                        .create_subscribe(
                            desc,
                            &participant.control.display_name,
                            offer,
                            status.audio,
                            status.video,
                        )
                        .await?;

                    websocket.subscribe_answer(desc, answer).await?;
                }
            }
            WebsocketEvent::PublishResponse { response } => {
                self.media.on_publish_response(response).await?;
            }
            WebsocketEvent::CandidateReceived { candidate, target } => {
                self.media.on_sdp_candidate(candidate, target).await?;
            }
            WebsocketEvent::EndOfCandidate { target } => {
                self.media.on_sdp_end_of_candidates(target).await?;
            }
            WebsocketEvent::Joined(participant) => {
                log::debug!("Participant {} joined!", participant.id);

                if participant.media.video.is_some() {
                    websocket
                        .subscribe(
                            MediaDescriptor {
                                participant_id: participant.id,
                                media_type: MediaSessionType::Video,
                            },
                            self.media.with_video,
                        )
                        .await?;
                }

                self.participants.push(participant);
            }
            WebsocketEvent::Update(updated) => {
                if let Some(current) = self.participants.iter_mut().find(|p| p.id == updated.id) {
                    let current_state = current.media.video;
                    let updated_state = updated.media.video;

                    Self::update_subscriptions_for_media_session_type(
                        &mut self.media,
                        websocket,
                        &*current,
                        &updated,
                        MediaSessionType::Video,
                        current_state,
                        updated_state,
                    )
                    .await?;

                    // update participant state
                    *current = updated;
                } else {
                    log::warn!("received update for unknown participant");
                }
            }
            WebsocketEvent::Left(assoc) => {
                if let Some(i) = self.participants.iter().position(|p| p.id == assoc.id) {
                    let participant = self.participants.remove(i);

                    if participant.media.video.is_some() {
                        self.media
                            .remove_subscribe(MediaDescriptor {
                                participant_id: assoc.id,
                                media_type: MediaSessionType::Video,
                            })
                            .await?;
                    }
                }
            }
            WebsocketEvent::Receiving(media_session_type, receiving) => {
                if self.publish_complete_pending && receiving {
                    self.publish_complete_pending = false;
                    websocket
                        .publish_complete(self.media.with_video, media_session_type)
                        .await?;
                }
            }
            WebsocketEvent::RequestMute { .. } => {
                if !self.muted {
                    self.muted = true;
                    websocket
                        .send_mute(
                            self.muted,
                            self.muted && (!self.media.with_video),
                            MediaSessionType::Video,
                        )
                        .await?;
                    self.track_controller.play_track(Track::ModeratorMuted);
                }
            }
            WebsocketEvent::SessionEnded { .. } => {
                self.close_websocket().await?;
                self.track_controller
                    .play_track_and_respond(Track::ConferenceClosed, closed_finished_responder);
            }
            WebsocketEvent::Disconnected => {
                self.state = State::Quitting;
            }
        }

        Ok(())
    }

    async fn update_subscriptions_for_media_session_type(
        media: &mut MediaPipeline,
        websocket: &mut Websocket,
        current: &Participant,
        updated: &Participant,
        media_session_type: MediaSessionType,
        current_state: Option<MediaSessionState>,
        updated_state: Option<MediaSessionState>,
    ) -> Result<(), anyhow::Error> {
        match (&current_state, &updated_state) {
            (Some(_), None) => {
                log::debug!("Participant {} no longer publishing!", updated.id);

                media
                    .remove_subscribe(MediaDescriptor {
                        participant_id: current.id,
                        media_type: media_session_type,
                    })
                    .await?;
            }
            (None, Some(_)) => {
                log::debug!("Participant {} is now publishing!", updated.id);

                websocket
                    .subscribe(
                        MediaDescriptor {
                            participant_id: current.id,
                            media_type: media_session_type,
                        },
                        media.with_video,
                    )
                    .await?;
            }
            (Some(old_status), Some(new_status)) if old_status != new_status => {
                log::debug!("Participant {} updated their mute status", updated.id);

                media.update_subscribe(
                    MediaDescriptor {
                        participant_id: current.id,
                        media_type: media_session_type,
                    },
                    new_status.audio,
                    new_status.video,
                )?;
            }
            _ => {
                // ignore
            }
        }

        Ok(())
    }

    async fn join(&mut self) -> Result<()> {
        let ticket = self
            .http_client
            .start(&self.settings.controller, &self.dtmf_id, &self.dtmf_pw)
            .await?;

        let mut websocket = Websocket::connect(&self.settings.controller, ticket).await?;

        let name = self
            .name
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("Anonymous");

        match websocket.join(name).await? {
            JoinState::Joined(participants) => {
                self.participants = participants;
            }
            JoinState::InWaitingRoom => {
                self.track_controller.play_track(Track::EnteredWaitingRoom);
                websocket.wait_until_accepted_into_waiting_room().await?;

                self.participants = websocket.enter_from_waiting_room().await?;
            }
        }

        self.websocket = Some(websocket);

        Ok(())
    }
}

async fn websocket_receive(websocket: &mut Option<Websocket>) -> Result<WebsocketEvent> {
    if let Some(websocket) = websocket {
        websocket.receive().await
    } else {
        pending().await
    }
}

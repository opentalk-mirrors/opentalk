// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::http::{HttpClient, InvalidCredentials};
use crate::media::{HandleEvent, MediaTaskHandle, Target, Track, TrackController};
use crate::settings::Settings;
use crate::websocket::{JoinState, Participant, Websocket, WebsocketEvent};
use anyhow::{bail, Result};
use bytesstr::BytesStr;
use sip_types::header::typed::ContentType;
use sip_types::Code;
use sip_ua::invite::session::{Event, ReInviteReceived, Session};
use std::fmt::Write;
use std::future::pending;
use std::sync::Arc;
use tokio::sync::broadcast;

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
    media: MediaTaskHandle,
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
    pub fn new(
        settings: Arc<Settings>,
        http_client: Arc<HttpClient>,
        name: Option<BytesStr>,
        sip: Session,
        media: MediaTaskHandle,
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

        while matches!(self.state, State::Running) {
            tokio::select! {
                event = self.sip.drive() => {
                    match event? {
                        Event::RefreshNeeded(event) => event.process_default().await?,
                        Event::ReInviteReceived(event) => {
                            if let Err(e) = handle_reinvite(&mut self.media, event).await {
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
                event = self.media.poll_event() => {
                    if let Err(e) = self.handle_media_event(event, welcome_finished_channel_tx.clone()).await {
                        log::error!("failed to handle media event, {:?}", e);
                        self.state = State::Quitting;
                    }
                }
                event = websocket_receive(&mut self.websocket) => {
                    if let Err(e) = self.handle_websocket_event(event).await {
                        log::error!("failed to handle websocket event, {:?}", e);
                        self.state = State::Quitting;
                    }
                }
                _ = welcome_finished_channel_rx.recv() => {
                    // Send publish command to media task
                    self.media.send_create_publish()?;

                    // Websocket is set on join
                    let websocket = self.websocket.as_mut().unwrap();

                    // Subscribe to all participants that currently publish
                    for participant in &self.participants {
                        if participant.control.left_at.is_none()
                            && participant.media.video.is_some()
                        {
                            websocket.subscribe(participant.id).await?;
                        }
                    }
                }
                _ = self.shutdown.recv() => {
                    self.state = State::Quitting;
                }
            }
        }

        if matches!(self.state, State::Quitting) {
            self.sip.terminate().await?;
        }

        if let Some(websocket) = &mut self.websocket {
            websocket.close().await?;
        }

        Ok(())
    }

    async fn handle_media_event(
        &mut self,
        event: HandleEvent,
        welcome_finished_responder: broadcast::Sender<()>,
    ) -> Result<()> {
        match event {
            HandleEvent::DtmfDigit(digit) if self.dtmf_pw.len() < 10 => {
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
            HandleEvent::DtmfDigit(digit) => {
                if digit == 0 {
                    self.track_controller.play_track(Track::Silence)
                } else if digit == 1 {
                    if let Some(websocket) = &mut self.websocket {
                        self.muted = !self.muted;
                        self.media.send_publish_mute(self.muted)?;
                        self.track_controller.play_track(if self.muted {
                            Track::Muted
                        } else {
                            Track::Unmuted
                        });
                        websocket.send_audio_mute(self.muted).await?;
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
            HandleEvent::Error => {
                self.state = State::Quitting;
            }
            HandleEvent::IceCandidate {
                target,
                candidate,
                mline_index,
            } => {
                if let Some(websocket) = &mut self.websocket {
                    match target {
                        Target::Publish => {
                            websocket
                                .send_ice_candidate(None, candidate, mline_index)
                                .await?
                        }
                        Target::Subscribe(uuid) => {
                            websocket
                                .send_ice_candidate(Some(uuid), candidate, mline_index)
                                .await?
                        }
                    }
                }
            }
            HandleEvent::NoMoreIceCandidates(target) => {
                if let Some(websocket) = &mut self.websocket {
                    match target {
                        Target::Publish => websocket.publish_end_of_candidates().await?,
                        Target::Subscribe(uuid) => {
                            websocket.subscribe_end_of_candidates(uuid).await?;
                        }
                    }
                }
            }
            HandleEvent::PublishOffer(offer) => {
                if let Some(websocket) = &mut self.websocket {
                    websocket.publish(&offer).await?;
                }
            }
            HandleEvent::SubscribeAnswer { uuid, answer } => {
                if let Some(websocket) = &mut self.websocket {
                    websocket.subscribe_answer(uuid, answer).await?;
                }
            }
        }

        Ok(())
    }

    async fn handle_websocket_event(&mut self, event: Result<WebsocketEvent>) -> Result<()> {
        let websocket = self.websocket.as_mut().expect("websocket must be set");

        match event? {
            WebsocketEvent::SubscribeOffer { offer, uuid } => {
                self.media.send_create_subscribe(uuid, offer)?;
            }
            WebsocketEvent::PublishResponse { response } => {
                self.media.send_publish_response(response)?;
            }
            WebsocketEvent::Joined(participant) => {
                log::debug!("Participant {} joined!", participant.id);

                if participant.media.video.is_some() {
                    websocket.subscribe(participant.id).await?;
                }

                self.participants.push(participant);
            }
            WebsocketEvent::Update(updated) => {
                if let Some(current) = self.participants.iter_mut().find(|p| p.id == updated.id) {
                    match (&current.media.video, &updated.media.video) {
                        (Some(_), None) => {
                            log::debug!("Participant {} no longer publishing!", updated.id);

                            self.media.send_remove_subscribe(updated.id)?;
                        }
                        (None, Some(_)) => {
                            log::debug!("Participant {} is now publishing!", updated.id);

                            websocket.subscribe(updated.id).await?;
                        }
                        _ => {
                            // ignore
                        }
                    }

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
                        self.media.send_remove_subscribe(participant.id)?;
                    }
                }
            }
            WebsocketEvent::Receiving(receiving) => {
                if self.publish_complete_pending && receiving {
                    self.publish_complete_pending = false;
                    websocket.publish_complete().await?;
                }
            }
            WebsocketEvent::RequestMute { .. } => {
                if !self.muted {
                    self.muted = true;
                    self.media.send_publish_mute(self.muted)?;
                    self.track_controller.play_track(Track::ModeratorMuted);
                    websocket.send_audio_mute(self.muted).await?;
                }
            }
            WebsocketEvent::Disconnected => {
                self.state = State::Quitting;
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

async fn handle_reinvite(media: &mut MediaTaskHandle, event: ReInviteReceived<'_>) -> Result<()> {
    let request = &event.invite;

    let body_is_not_sdp = request
        .headers
        .get_named()
        .map(|content_type: ContentType| content_type.0 != "application/sdp")
        .unwrap_or_default();

    if body_is_not_sdp {
        let response = event
            .session
            .endpoint
            .create_response(request, Code::NOT_ACCEPTABLE, None);

        event.transaction.respond_failure(response).await?;

        bail!("failed to handle re-invite unknown content-type");
    }

    let body = request.body.clone();

    match media.send_sip_sdp_reinvite(body).await {
        Ok(answer) => {
            let mut response = event
                .session
                .endpoint
                .create_response(request, Code::OK, None);

            response.msg.body = answer.into();

            event.respond_success(response).await?;

            Ok(())
        }
        Err(_) => {
            log::error!("failed to handle sdp update");

            let response =
                event
                    .session
                    .endpoint
                    .create_response(request, Code::SERVER_INTERNAL_ERROR, None);

            event.transaction.respond_failure(response).await?;

            Ok(())
        }
    }
}

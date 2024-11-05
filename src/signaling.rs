// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::http::{HttpClient, InvalidCredentials};
use crate::media::{MediaEvent, MediaPipeline, Track, TrackController};
use crate::settings::Settings;
use crate::sip::{handle_reinvite, SessionState};
use crate::websocket::{JoinState, Participant, Websocket, WebsocketEvent};
use anyhow::{Context, Result};
use bytesstr::BytesStr;
use opentalk_compositor::{Mixer, MixerParameters};
use sip_ua::invite::session::{Event, Session};
use std::fmt::Write;
use std::future::pending;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{sleep, Duration};

const PIN_ENTRY_TIMEOUT_INTERVAL: Duration = Duration::from_secs(5);

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

    /// Timeout handler
    timeout_sender: tokio::sync::mpsc::Sender<()>,
    timeout_receiver: tokio::sync::mpsc::Receiver<()>,
    timeout_task_handle: Option<tokio::task::JoinHandle<()>>,
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
        let (timeout_sender, timeout_receiver) = mpsc::channel(1);

        Self {
            settings,
            http_client,
            name,
            sip,
            websocket: None,
            media,
            media_session,
            track_controller,
            state: State::Running,
            dtmf_id: String::new(),
            dtmf_pw: String::new(),
            muted: false,
            hand_raised: false,
            participants: Vec::new(),
            shutdown,
            timeout_sender,
            timeout_receiver,
            timeout_task_handle: None,
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
                    match handle_sip_event(event?, &mut self.media, &mut self.media_session).await {
                        Ok(Some(state)) => self.state = state,
                        Ok(None) => {}
                        Err(e) => {
                            log::error!("failed to handle sip event, {:?}", e);
                            self.state = State::Quitting;
                        }
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
                    }
                }
                _ = self.timeout_receiver.recv() => {
                    log::debug!("pin entry timed out");
                    self.dtmf_pw.clear();
                    self.track_controller.play_track(Track::InputInvalid);
                }
                _ = welcome_finished_channel_rx.recv() => {
                    self.join().await.context("Failed to join the room")?;
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

        if let Some(compositor) = &mut self.media.compositor {
            let p = compositor.local_participant();

            for track in p.track_publications().into_keys() {
                if let Err(e) = p.unpublish_track(&track).await {
                    log::error!("Failed to unpublish track, {e}");
                }
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

                            // Cancel the previous timeout if it exists
                            if let Some(task_handle) = self.timeout_task_handle.take() {
                                task_handle.abort();
                            }

                            if self.dtmf_pw.len() == 10 {
                                match self.connect_websocket().await {
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
                            } else {
                                // Start a new timeout
                                let timeout_sender = self.timeout_sender.clone();

                                let task_handle = tokio::spawn(async move {
                                    sleep(PIN_ENTRY_TIMEOUT_INTERVAL).await;
                                    let _ = timeout_sender.send(()).await;
                                });

                                self.timeout_task_handle = Some(task_handle);
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
                    self.set_publish_mute(!self.muted);

                    self.track_controller.play_track(if self.muted {
                        Track::Muted
                    } else {
                        Track::Unmuted
                    });
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
        }

        Ok(())
    }

    async fn handle_websocket_event(
        &mut self,
        event: Result<WebsocketEvent>,
        closed_finished_responder: broadcast::Sender<()>,
    ) -> Result<()> {
        match event? {
            WebsocketEvent::Joined(participant) => {
                log::debug!("Participant {} joined!", participant.id);

                if let Some(compositor) = &mut self.media.compositor {
                    compositor
                        .add_participant(
                            participant.id.to_string().into(),
                            participant.control.display_name.clone(),
                        )
                        .await;
                }

                self.participants.push(participant);
            }
            WebsocketEvent::Update(updated) => {
                let Some(current) = self.participants.iter_mut().find(|p| p.id == updated.id)
                else {
                    log::error!("Got update for unknown participant {:?}", updated.id);
                    return Ok(());
                };

                if let Some(compositor) = &mut self.media.compositor {
                    if current.control.display_name != updated.control.display_name {
                        compositor
                            .add_participant(
                                current.id.to_string().into(),
                                updated.control.display_name.clone(),
                            )
                            .await;
                    }
                }

                *current = updated;
            }
            WebsocketEvent::Left(assoc) => {
                self.participants.retain(|p| p.id != assoc.id);

                if let Some(compositor) = &mut self.media.compositor {
                    compositor
                        .remove_participant(&assoc.id.to_string().into())
                        .await;
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

    /// Connect to websocket & verify that the credentials are correct
    async fn connect_websocket(&mut self) -> Result<()> {
        let ticket = self
            .http_client
            .start(&self.settings.controller, &self.dtmf_id, &self.dtmf_pw)
            .await?;

        self.websocket = Some(Websocket::connect(&self.settings.controller, ticket).await?);

        Ok(())
    }

    /// Join the room after the welcome message has been played or skipped
    async fn join(&mut self) -> Result<()> {
        let websocket = self
            .websocket
            .as_mut()
            .context("Tried to join without websocket")?;

        let name = self.name.as_ref().map_or("Anonymous", |s| s.as_str());

        let join_result = match websocket.join(name).await? {
            JoinState::Joined(join_result) => join_result,
            JoinState::InWaitingRoom => {
                self.track_controller.play_track(Track::EnteredWaitingRoom);

                let mut wait_until_accepted =
                    std::pin::pin!(websocket.wait_until_accepted_into_waiting_room());

                loop {
                    tokio::select! {
                        join_result = &mut wait_until_accepted => {
                            // Participant got accepted, break out of loop
                            break join_result?;
                        }
                        sip_event = self.sip.drive() => {
                            if let Some(state) = handle_sip_event(sip_event?, &mut self.media, &mut self.media_session).await? {
                                self.state = state;
                                return Ok(())
                            }
                        }
                    }
                }
            }
        };

        self.participants = join_result.participants;

        let mut compositor = Mixer::new(MixerParameters {
            clock_format: self.settings.clock_format.clone(),
            livekit_url: join_result.livekit_url,
            livekit_token: join_result.livekit_token,
            auto_subscribe: false,
        })
        .await
        .context("Failed to create Mixer")?;

        for participant in &self.participants {
            if participant.control.left_at.is_some() {
                continue;
            };

            compositor
                .add_participant(
                    participant.id.to_string().into(),
                    participant.control.display_name.clone(),
                )
                .await;
        }

        self.media
            .add_compositor(compositor)
            .await
            .context("Failed to add compositor to media pipeline")?;

        Ok(())
    }

    fn set_publish_mute(&mut self, muted: bool) {
        self.muted = muted;

        if let Some(compositor) = &mut self.media.compositor {
            for track in compositor
                .local_participant()
                .track_publications()
                .into_values()
            {
                if self.muted {
                    track.mute();
                } else {
                    track.unmute();
                }
            }
        }
    }
}

async fn handle_sip_event(
    event: Event<'_>,
    media: &mut MediaPipeline,
    media_session: &mut SessionState,
) -> Result<Option<State>> {
    match event {
        Event::RefreshNeeded(event) => event.process_default().await?,
        Event::ReInviteReceived(event) => {
            if let Err(e) = handle_reinvite(media, media_session, event).await {
                log::error!("failed to handle re-invite, {:?}", e);
                return Ok(Some(State::Quitting));
            }
        }
        Event::Bye(event) => {
            event.process_default().await?;
            return Ok(Some(State::Terminated));
        }
        Event::Terminated => {
            return Ok(Some(State::Terminated));
        }
    }

    Ok(None)
}

async fn websocket_receive(websocket: &mut Option<Websocket>) -> Result<WebsocketEvent> {
    if let Some(websocket) = websocket {
        websocket.receive().await
    } else {
        pending().await
    }
}

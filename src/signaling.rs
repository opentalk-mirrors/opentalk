// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::http::{HttpClient, InvalidCredentials};
use crate::media::{MediaEvent, MediaPipeline, Track, TrackController};
use crate::settings::Settings;
use crate::sip::{handle_reinvite, SessionState};
use crate::websocket::{JoinState, JoinSuccess, Participant, Websocket, WebsocketEvent};
use anyhow::{Context, Result};
use bytes::Bytes;
use bytesstr::BytesStr;
use opentalk_compositor::livekit::track::TrackKind;
use opentalk_compositor::livekit::DisconnectReason;
use opentalk_compositor::{Mixer, MixerParameters};
use sip_types::{Method, Name};
use sip_ua::invite::session::{Event, Session};
use std::fmt::Write;
use std::future::pending;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{sleep, Duration, Sleep};
use types_signaling_livekit::event::LiveKitEvent;

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
    last_fast_picture_update_request_sent: Option<Instant>,

    /// Track controller to play audio tracks on demand to the SIP user
    track_controller: TrackController,

    /// Current state of the signaling
    state: State,

    /// DTMF ID is set before connecting the websocket
    dtmf_id: String,

    /// DTMF ID is set before connecting the websocket
    dtmf_pw: String,

    /// Is the client muted
    audio_muted: bool,
    video_muted: bool,

    /// has the client the hand raised
    hand_raised: bool,

    /// List of participants inside the room,
    /// unused before initiating the websocket connection
    participants: Vec<Participant>,

    /// Application shutdown signal
    shutdown: broadcast::Receiver<()>,

    /// Timeout handler
    pin_entry_timeout_sender: tokio::sync::mpsc::Sender<()>,
    pin_entry_timeout_receiver: tokio::sync::mpsc::Receiver<()>,
    pin_entry_timeout_task_handle: Option<tokio::task::JoinHandle<()>>,

    /// Grace period after being disconnected from livekit, since we might be moved into the waiting room
    livekit_disconnection_grace_period: Option<Pin<Box<Sleep>>>,
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
            last_fast_picture_update_request_sent: None,
            track_controller,
            state: State::Running,
            dtmf_id: String::new(),
            dtmf_pw: String::new(),
            audio_muted: false,
            video_muted: false,
            hand_raised: false,
            participants: Vec::new(),
            shutdown,
            pin_entry_timeout_sender: timeout_sender,
            pin_entry_timeout_receiver: timeout_receiver,
            pin_entry_timeout_task_handle: None,
            livekit_disconnection_grace_period: None,
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

                    // Update pre-conference UI
                    self.media.ui_controller.set(self.dtmf_id.clone(), self.dtmf_pw.clone());
                }
                event = websocket_receive(&mut self.websocket) => {
                    if let Err(e) = self.handle_websocket_event(event, closed_finished_channel_tx.clone()).await {
                        log::error!("failed to handle websocket event, {:?}", e);
                    }
                }
                _ = self.pin_entry_timeout_receiver.recv() => {
                    log::debug!("pin entry timed out");
                    self.dtmf_pw.clear();
                    self.track_controller.play_track(self.settings.language, Track::InputInvalid);
                }
                _ = welcome_finished_channel_rx.recv() => {
                    self.join().await.context("Failed to join the room")?;
                }
                _ = closed_finished_channel_rx.recv() => {
                    self.state = State::Quitting;
                }
                _ = opt_timeout(&mut self.livekit_disconnection_grace_period) => {

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
                                self.track_controller
                                    .play_track(self.settings.language, Track::WelcomePasscode);
                            }
                        } else {
                            let _ = write!(&mut self.dtmf_pw, "{}", digit);

                            // Cancel the previous timeout if it exists
                            if let Some(task_handle) = self.pin_entry_timeout_task_handle.take() {
                                task_handle.abort();
                            }

                            if self.dtmf_pw.len() == 10 {
                                match self.connect_websocket().await {
                                    Ok(_) => {
                                        self.track_controller.play_track_and_respond(
                                            self.settings.language,
                                            Track::WelcomeUsage,
                                            welcome_finished_responder,
                                        );
                                    }
                                    Err(e) if e.is::<InvalidCredentials>() => {
                                        self.dtmf_id.clear();
                                        self.dtmf_pw.clear();

                                        self.track_controller.play_track(
                                            self.settings.language,
                                            Track::InputInvalid,
                                        );
                                        return Ok(());
                                    }
                                    Err(e) => {
                                        log::error!("failed to join, {}", e);
                                        return Err(e);
                                    }
                                }
                            } else {
                                // Start a new timeout
                                let timeout_sender = self.pin_entry_timeout_sender.clone();

                                let task_handle = tokio::spawn(async move {
                                    sleep(PIN_ENTRY_TIMEOUT_INTERVAL).await;
                                    let _ = timeout_sender.send(()).await;
                                });

                                self.pin_entry_timeout_task_handle = Some(task_handle);
                            }
                        }
                    }
                    _ => {}
                }
            }
            MediaEvent::DtmfDigit(digit) => match digit {
                0 => self.track_controller.stop_current_track(),
                1 => {
                    self.toggle_mute(TrackKind::Audio).await;
                }
                2 => {
                    if let Some(websocket) = &mut self.websocket {
                        self.hand_raised = !self.hand_raised;
                        self.track_controller.play_track(
                            self.settings.language,
                            if self.hand_raised {
                                Track::On
                            } else {
                                Track::Off
                            },
                        );

                        websocket.send_hand_action(self.hand_raised).await?;
                    }
                }
                3 => {
                    if let Some(websocket) = &mut self.websocket {
                        websocket.set_consent(true).await?;
                    }
                }
                4 => {
                    self.toggle_mute(TrackKind::Video).await;
                }
                6 => {
                    if let Some(websocket) = &mut self.websocket {
                        websocket.set_consent(false).await?;
                    }
                }
                _ => {}
            },
            MediaEvent::Error => {
                self.state = State::Quitting;
            }
            MediaEvent::LocalTrackMuted(kind) => {
                let muted = match kind {
                    TrackKind::Audio => &mut self.audio_muted,
                    TrackKind::Video => &mut self.video_muted,
                };

                if !*muted {
                    *muted = true;

                    self.track_controller
                        .play_track(self.settings.language, Track::ModeratorMuted);
                }
            }
            MediaEvent::VideoDecoderError => {
                self.send_picture_update_request().await?;
            }
            MediaEvent::Disconnected(disconnect_reason) => {
                // When the participant was removed, he probably was moved into the waiting room.
                // Wait for the waiting room notification
                if disconnect_reason == DisconnectReason::ParticipantRemoved.into() {
                    self.livekit_disconnection_grace_period =
                        Some(Box::pin(sleep(Duration::from_secs(3))));
                } else {
                    self.state = State::Quitting;
                }
            }
        }

        Ok(())
    }

    async fn send_picture_update_request(&mut self) -> Result<()> {
        // Send a request at most every 5 seconds
        let now = Instant::now();
        if let Some(i) = self.last_fast_picture_update_request_sent {
            if i + Duration::from_secs(5) > now {
                return Ok(());
            }
        }
        self.last_fast_picture_update_request_sent = Some(now);

        let mut target_tp_info = self.sip.dialog.target_tp_info.lock().await;

        let mut request = self.sip.dialog.create_request(Method::INFO);
        request
            .headers
            .insert(Name::CONTENT_TYPE, "application/media_control+xml");
        request.body = Bytes::from_static(b"<?xml version=\"1.0\" encoding=\"utf-8\"?><media_control><vc_primitive><to_encoder><picture_fast_update></picture_fast_update></to_encoder></vc_primitive></media_control>\r\n");

        let mut tsx = self
            .sip
            .endpoint
            .send_request(request, &mut target_tp_info)
            .await?;

        tokio::spawn(async move {
            let _ = tsx.receive_final().await;
        });

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
                    compositor.add_participant(
                        &participant.id.to_string().into(),
                        participant.control.display_name.to_string(),
                    );
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
                        compositor.add_participant(
                            &current.id.to_string().into(),
                            updated.control.display_name.to_string(),
                        );
                    }
                }

                *current = updated;
            }
            WebsocketEvent::Left(assoc) => {
                self.participants.retain(|p| p.id != assoc.id);

                if let Some(compositor) = &mut self.media.compositor {
                    compositor.remove_participant(&assoc.id.to_string().into());
                }
            }
            WebsocketEvent::SessionEnded { .. } => {
                self.close_websocket().await?;
                self.track_controller.play_track_and_respond(
                    self.settings.language,
                    Track::ConferenceClosed,
                    closed_finished_responder,
                );
            }
            WebsocketEvent::MovedToWaitingRoom => {
                self.participants.clear();
                self.media.remove_compositor().await;

                if let Some(join_result) = self.wait_in_waiting_room_until_accepted().await? {
                    self.handle_join_result(join_result).await?;
                }
            }
            WebsocketEvent::LiveKitEvent(event) => match event {
                LiveKitEvent::MicrophoneRestrictionsEnabled(unrestricted_participants) => {
                    let websocket = self.websocket.as_mut().unwrap();

                    let is_unrestricted = unrestricted_participants
                        .unrestricted_participants
                        .contains(&websocket.id.expect("id must be set"));

                    // if is restricted, mute and play the track
                    if !is_unrestricted {
                        self.audio_muted = true;
                        self.track_controller.play_track(
                            self.settings.language,
                            Track::MicrophonesRestrictionEnabled,
                        );
                    }

                    self.media
                        .set_microphone_publish(is_unrestricted, self.audio_muted)
                        .await;
                }
                LiveKitEvent::MicrophoneRestrictionsDisabled => {
                    self.track_controller
                        .play_track(self.settings.language, Track::MicrophonesRestrictionLifted);

                    self.media
                        .set_microphone_publish(true, self.audio_muted)
                        .await;
                }
                _ => {}
            },
            WebsocketEvent::Disconnected => {
                self.state = State::Quitting;
            }
            WebsocketEvent::RecordingStarted => {
                self.track_controller
                    .play_track(self.settings.language, Track::ConsentInfo);
            }
            WebsocketEvent::RecordingPaused => {}
            WebsocketEvent::RecordingStopped => {}
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
                if let Some(join_result) = self.wait_in_waiting_room_until_accepted().await? {
                    join_result
                } else {
                    // We hung up, return
                    return Ok(());
                }
            }
        };

        self.handle_join_result(join_result).await
    }

    /// Wait in the waiting room until accepted or the call is hung up
    async fn wait_in_waiting_room_until_accepted(&mut self) -> Result<Option<JoinSuccess>> {
        self.media.ui_controller.set_waiting_room(true);

        let websocket = self.websocket.as_mut().unwrap();
        self.track_controller
            .play_track(self.settings.language, Track::WaitingRoomEntered);

        let mut wait_until_accepted =
            std::pin::pin!(websocket.wait_until_accepted_into_waiting_room());

        loop {
            tokio::select! {
                join_result = &mut wait_until_accepted => {
                    // Participant got accepted, break out of loop
                    return Ok(Some(join_result?));
                }
                sip_event = self.sip.drive() => {
                    if let Some(state) = handle_sip_event(sip_event?, &mut self.media, &mut self.media_session).await? {
                        // Got hung up, return
                        self.state = state;
                        return Ok(None)
                    }
                }
            }
        }
    }

    async fn handle_join_result(&mut self, join_result: JoinSuccess) -> Result<()> {
        self.participants = join_result.participants;

        let mut compositor = Mixer::new(MixerParameters {
            target_fps: self.media.target_fps as u16,
            clock_format: self.settings.clock_format.clone(),
            livekit_url: join_result
                .livekit
                .credentials
                .service_url
                .unwrap_or(join_result.livekit.credentials.public_url),
            livekit_token: join_result.livekit.credentials.token,
            auto_subscribe: false,
        })
        .await
        .context("Failed to create Mixer")?;

        for participant in &self.participants {
            if participant.control.left_at.is_some() {
                continue;
            };

            compositor.add_participant(
                &participant.id.to_string().into(),
                participant.control.display_name.to_string(),
            );
        }

        self.media
            .add_compositor(compositor)
            .await
            .context("Failed to add compositor to media pipeline")?;

        let microphone_enable = match join_result.livekit.microphone_restriction_state {
            types_signaling_livekit::MicrophoneRestrictionState::Disabled => true,
            types_signaling_livekit::MicrophoneRestrictionState::Enabled {
                unrestricted_participants,
            } => unrestricted_participants.contains(&join_result.id),
        };

        if !microphone_enable {
            self.track_controller
                .play_track(self.settings.language, Track::MicrophonesRestrictionEnabled);
        }

        self.media
            .set_microphone_publish(microphone_enable, self.audio_muted)
            .await;

        Ok(())
    }

    // Toggle the mute state of our local publications
    async fn toggle_mute(&mut self, kind: TrackKind) {
        // Do not toggle anything if there's nothing to toggle
        if kind == TrackKind::Audio && !self.media.is_microphone_published().await {
            self.track_controller
                .play_track(self.settings.language, Track::MicrophonesRestrictionEnabled);
            return;
        }

        if let Some(compositor) = &mut self.media.compositor {
            for track in compositor
                .local_participant()
                .track_publications()
                .into_values()
            {
                if track.kind() != kind {
                    continue;
                }

                let is_muted = match kind {
                    TrackKind::Audio => self.audio_muted,
                    TrackKind::Video => self.video_muted,
                };

                if is_muted {
                    track.unmute();
                } else {
                    track.mute();
                }
            }
        }

        let is_muted = match kind {
            TrackKind::Audio => {
                self.audio_muted = !self.audio_muted;
                self.audio_muted
            }
            TrackKind::Video => {
                self.video_muted = !self.video_muted;
                self.video_muted
            }
        };

        self.track_controller.play_track(
            self.settings.language,
            if is_muted { Track::On } else { Track::Off },
        );
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

async fn opt_timeout(timeout: &mut Option<Pin<Box<Sleep>>>) {
    match timeout {
        Some(sleep) => sleep.await,
        None => pending().await,
    }
}

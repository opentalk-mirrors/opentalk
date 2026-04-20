// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::media::{MediaEvent, MediaPipeline, Track, TrackController};
use crate::settings::Settings;
use crate::sip::{SessionState, handle_reinvite, refresh_invite_session};
use anyhow::{Context, Result, bail};
use bytes::Bytes;
use bytesstr::BytesStr;
use opentalk_client_signaling::client::JoinResult;
use opentalk_client_signaling::http::StartCallInError;
use opentalk_client_signaling::opentalk_roomserver_types::breakout::event::BreakoutEvent;
use opentalk_client_signaling::opentalk_roomserver_types::connection_id::ConnectionId;
use opentalk_client_signaling::opentalk_roomserver_types_livekit::MicrophoneRestrictionState;
use opentalk_client_signaling::types::common::users::DisplayName;
use opentalk_client_signaling::types::signaling::ParticipantId;
use opentalk_client_signaling::{
    CallInConnectToken, DisconnectReason, OpenTalkClient, OpenTalkEvent, OpenTalkLiveKitEvent,
    OpenTalkRecordingEvent, Participant, Room,
};
use opentalk_compositor::livekit::{DisconnectReason as LiveKitDisconnectReason, track::TrackKind};
use opentalk_compositor::{Mixer, MixerParameters};
use sip_types::{Method, Name};
use sip_ua::invite::session::{InviteSession, InviteSessionEvent};
use std::fmt::Write;
use std::future::pending;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{Duration, Sleep, sleep};

// DTMF digit bindings
const DTMF_STOP_CURRENT_TRACK: u8 = 0;
const DTMF_TOGGLE_AUDIO_MUTE: u8 = 1;
const DTMF_TOGGLE_VIDEO_MUTE: u8 = 4;
const DTMF_TOGGLE_HAND_RAISE: u8 = 2;
const DTMF_ACCEPT_RECORDING: u8 = 3;
const DTMF_REJECT_RECORDING: u8 = 6;

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

    opentalk_client: Arc<OpenTalkClient>,

    /// Name of the user (usually the phone number or Anonymous)
    name: Option<BytesStr>,

    /// The SIP session
    sip: InviteSession,

    /// Connect token which will be held while the announcement for joining a room plays
    connect_token: Option<CallInConnectToken>,
    /// OpenTalk room connection
    opentalk_room: Option<Room>,

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

    // Tracks if the user has been informed of an active recording
    have_active_running_recording: bool,
    recording_consent_state: RecordingConsentState,

    /// Application shutdown signal
    shutdown: broadcast::Receiver<()>,

    /// Timeout handler
    pin_entry_timeout_sender: tokio::sync::mpsc::Sender<()>,
    pin_entry_timeout_receiver: tokio::sync::mpsc::Receiver<()>,
    pin_entry_timeout_task_handle: Option<tokio::task::JoinHandle<()>>,

    /// Grace period after being disconnected from livekit, since we might be moved into the waiting room
    livekit_disconnection_grace_period: Option<Pin<Box<Sleep>>>,
}

enum RecordingConsentState {
    None,
    WaitingForConsent,
    Accepted,
    Rejected,
}

impl Signaling {
    /// Create a new Signaling from a newly established SIP session
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        settings: Arc<Settings>,
        opentalk_client: Arc<OpenTalkClient>,
        name: Option<BytesStr>,
        sip: InviteSession,
        media: MediaPipeline,
        media_session: SessionState,
        track_controller: TrackController,
        shutdown: broadcast::Receiver<()>,
    ) -> Self {
        let (timeout_sender, timeout_receiver) = mpsc::channel(1);

        Signaling {
            settings,
            opentalk_client,
            name,
            sip,
            connect_token: None,
            opentalk_room: None,
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
            have_active_running_recording: false,
            recording_consent_state: RecordingConsentState::None,
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
                event = self.sip.run() => {
                    match handle_sip_event(&mut self.sip, event?, &mut self.media, &mut self.media_session).await {
                        Ok(Some(state)) => self.state = state,
                        Ok(None) => {}
                        Err(e) => {
                            log::error!("failed to handle sip event, {e:?}");
                            self.state = State::Quitting;
                        }
                    }
                }
                event = self.media.wait_for_event() => {
                    if let Err(e) = self.handle_media_event(event, welcome_finished_channel_tx.clone()).await {
                        log::error!("failed to handle media event, {e:?}");
                        self.state = State::Quitting;
                    }

                    // Update pre-conference UI
                    self.media.ui_controller.set(self.dtmf_id.clone(), self.dtmf_pw.clone());
                }
                event = opentalk_room_receive(&mut self.opentalk_room) => {
                    if let Err(e) = Box::pin(self.handle_opentalk_room_event(event, closed_finished_channel_tx.clone())).await {
                        log::error!("failed to handle websocket event, {e:?}");
                    }
                }
                _ = self.pin_entry_timeout_receiver.recv() => {
                    log::debug!("pin entry timed out");
                    self.dtmf_pw.clear();
                    self.track_controller.play_track(self.settings.language, Track::InputInvalid);
                }
                _ = welcome_finished_channel_rx.recv() => {
                    if let Err(e) = self.connect_to_conference().await  {
                        log::error!("Failed to join conference, {e:?}");
                        self.state = State::Quitting;
                    }
                }
                _ = closed_finished_channel_rx.recv() => {
                    self.state = State::Quitting;
                }
                () = opt_timeout(&mut self.livekit_disconnection_grace_period) => {

                }
                _ = self.shutdown.recv() => {
                    self.state = State::Quitting;
                }
            }
        }

        if matches!(self.state, State::Quitting) {
            self.sip.terminate().await?;
        }

        if let Some(compositor) = self.media.compositor.take() {
            let p = compositor.local_participant();

            for track in p.track_publications().into_keys() {
                if let Err(e) = p.unpublish_track(&track).await {
                    log::error!("Failed to unpublish track, {e}");
                }
            }
        }

        self.close_websocket().await?;

        Ok(())
    }

    async fn close_websocket(&mut self) -> Result<()> {
        if let Some(room) = self.opentalk_room.take() {
            room.close().await?;
        }

        Ok(())
    }

    #[expect(clippy::too_many_lines)]
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
                    0..=9 => {
                        if self.dtmf_id.len() < 10 {
                            let _ = write!(&mut self.dtmf_id, "{digit}");

                            if self.dtmf_id.len() == 10 {
                                self.track_controller
                                    .play_track(self.settings.language, Track::WelcomePasscode);
                            }
                        } else {
                            let _ = write!(&mut self.dtmf_pw, "{digit}");

                            // Cancel the previous timeout if it exists
                            if let Some(task_handle) = self.pin_entry_timeout_task_handle.take() {
                                task_handle.abort();
                            }

                            if self.dtmf_pw.len() == 10 {
                                match self.start_connect_to_conference().await {
                                    Ok(()) => {
                                        self.track_controller.play_track_and_respond(
                                            self.settings.language,
                                            Track::WelcomeUsage,
                                            welcome_finished_responder,
                                        );
                                    }
                                    Err(e) => {
                                        if let Some(StartCallInError::InvalidIdOrPin) =
                                            e.downcast_ref::<StartCallInError>()
                                        {
                                            self.dtmf_id.clear();
                                            self.dtmf_pw.clear();

                                            self.track_controller.play_track(
                                                self.settings.language,
                                                Track::InputInvalid,
                                            );
                                            return Ok(());
                                        } else {
                                            log::error!("failed to join, {e}");
                                            return Err(e);
                                        }
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
            MediaEvent::DtmfDigit(digit) => {
                // Only allow consent bindings if awaiting a decision
                if matches!(
                    self.recording_consent_state,
                    RecordingConsentState::WaitingForConsent
                ) && !matches!(digit, DTMF_ACCEPT_RECORDING | DTMF_REJECT_RECORDING)
                {
                    self.track_controller
                        .play_track(self.settings.language, Track::ConsentInvalid);
                    return Ok(());
                }

                match digit {
                    DTMF_STOP_CURRENT_TRACK => self.track_controller.stop_current_track(),
                    DTMF_TOGGLE_AUDIO_MUTE => {
                        self.toggle_audio_mute().await;
                    }
                    DTMF_TOGGLE_VIDEO_MUTE => {
                        self.toggle_video_mute(true);
                    }
                    DTMF_TOGGLE_HAND_RAISE => {
                        if let Some(opentalk_room) = &mut self.opentalk_room {
                            self.hand_raised = !self.hand_raised;
                            self.track_controller.play_track(
                                self.settings.language,
                                if self.hand_raised {
                                    Track::On
                                } else {
                                    Track::Off
                                },
                            );

                            opentalk_room.set_hand_raised(self.hand_raised).await?;
                        }
                    }
                    DTMF_REJECT_RECORDING => {
                        // Reject recording
                        let Some(opentalk_room) = &mut self.opentalk_room else {
                            return Ok(());
                        };

                        if matches!(
                            self.recording_consent_state,
                            RecordingConsentState::WaitingForConsent
                                | RecordingConsentState::Accepted
                        ) {
                            opentalk_room.set_recording_consent(false).await?;
                            self.track_controller
                                .play_track(self.settings.language, Track::ConsentReject);
                            self.recording_consent_state = RecordingConsentState::Rejected;
                            self.evaluate_microphone_publish_state().await;
                            if !self.video_muted {
                                self.toggle_video_mute(false);
                            }
                        }
                    }
                    DTMF_ACCEPT_RECORDING => {
                        // Accept recording
                        let Some(opentalk_room) = &mut self.opentalk_room else {
                            return Ok(());
                        };

                        if matches!(
                            self.recording_consent_state,
                            RecordingConsentState::WaitingForConsent
                                | RecordingConsentState::Rejected
                        ) {
                            opentalk_room.set_recording_consent(true).await?;
                            self.track_controller
                                .play_track(self.settings.language, Track::ConsentAccept);
                            self.recording_consent_state = RecordingConsentState::Accepted;
                            self.evaluate_microphone_publish_state().await;
                        }
                    }
                    _ => {}
                }
            }
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
                if disconnect_reason == LiveKitDisconnectReason::ParticipantRemoved.into() {
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
        if let Some(i) = self.last_fast_picture_update_request_sent
            && i + Duration::from_secs(5) > now
        {
            return Ok(());
        }
        self.last_fast_picture_update_request_sent = Some(now);

        let mut target_tp_info = self.sip.dialog.target_tp_info.lock().await;

        let mut request = self.sip.dialog.create_request(Method::INFO, None);
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

    async fn handle_opentalk_room_event(
        &mut self,
        event: Result<OpenTalkEvent>,
        closed_finished_responder: broadcast::Sender<()>,
    ) -> Result<()> {
        let opentalk_room = self
            .opentalk_room
            .as_mut()
            .context("Called handle_opentalk_room_event without a room set")?;

        match event? {
            OpenTalkEvent::ParticipantJoined(participant) => {
                self.handle_participant_joined(&participant);
            }
            OpenTalkEvent::ParticipantUpdated { previous, updated } => {
                if let Some(compositor) = &mut self.media.compositor
                    && previous.display_name != updated.display_name
                {
                    compositor.add_participant(
                        &updated.id.to_string().into(),
                        updated.display_name.to_string(),
                    );
                }
            }
            OpenTalkEvent::ParticipantLeft(participant_id, connection_id) => {
                self.handle_participant_left(participant_id, connection_id);
            }
            OpenTalkEvent::MovedToWaitingRoom => {
                self.media.remove_compositor().await;
            }

            OpenTalkEvent::WaitingRoomAccepted => {
                opentalk_room
                    .enter_from_waiting_room()
                    .await
                    .context("Failed to enter conference from waiting room")?;

                self.media.ui_controller.set_waiting_room(false);

                self.handle_join_result().await?;
            }
            OpenTalkEvent::LiveKit(event) => match event {
                OpenTalkLiveKitEvent::MicrophoneRestrictionEnabled {
                    unrestricted_participants,
                } => {
                    let is_unrestricted =
                        unrestricted_participants.contains(&opentalk_room.participant_id());

                    // if is restricted, mute and play the track
                    if !is_unrestricted {
                        self.audio_muted = true;
                        self.track_controller.play_track(
                            self.settings.language,
                            Track::MicrophonesRestrictionEnabled,
                        );
                    }

                    self.evaluate_microphone_publish_state().await;
                }
                OpenTalkLiveKitEvent::MicrophoneRestrictionDisabled => {
                    self.track_controller
                        .play_track(self.settings.language, Track::MicrophonesRestrictionLifted);

                    self.evaluate_microphone_publish_state().await;
                }
                _ => {}
            },
            OpenTalkEvent::Disconnected(reason) => {
                match reason {
                    DisconnectReason::WebsocketClosed => {
                        log::error!("Unexpected close of websocket to room");
                        self.state = State::Quitting;
                    }
                    DisconnectReason::Kicked | DisconnectReason::Banned => {
                        self.track_controller.play_track_and_respond(
                            self.settings.language,
                            Track::ModeratorKicked,
                            closed_finished_responder,
                        );
                    }
                    DisconnectReason::DebriefingStarted => {
                        self.track_controller.play_track_and_respond(
                            self.settings.language,
                            Track::ConferenceClosed,
                            closed_finished_responder,
                        );
                    }
                }

                self.opentalk_room = None;
            }
            OpenTalkEvent::Recording(event) => self.handle_recording_event(event).await?,
            OpenTalkEvent::RecordingService(..) => {
                // ignore
            }
            OpenTalkEvent::Breakout(breakout_event) => {
                self.handle_breakout_event(breakout_event)?
            }
        }

        Ok(())
    }

    async fn handle_recording_event(&mut self, event: OpenTalkRecordingEvent) -> Result<()> {
        let opentalk_room = self
            .opentalk_room
            .as_mut()
            .context("Called handle_recording_event without a room set")?;

        match event {
            OpenTalkRecordingEvent::StreamTargetsUpdated { .. }
            | OpenTalkRecordingEvent::RecordingStateUpdate { .. } => {
                let prev_recording_active = self.have_active_running_recording;
                let now_recording_active = is_any_recording_active(opentalk_room);

                // check if a recording was started
                if !prev_recording_active && now_recording_active {
                    self.track_controller
                        .play_track(self.settings.language, Track::ConsentInfo);
                    self.recording_consent_state = RecordingConsentState::WaitingForConsent;
                }

                // check if all streams are now inactive
                if prev_recording_active && !now_recording_active {
                    match &self.recording_consent_state {
                        RecordingConsentState::None => {}
                        RecordingConsentState::WaitingForConsent
                        | RecordingConsentState::Accepted => {
                            self.track_controller
                                .play_track(self.settings.language, Track::RecordingStoppedAccept);
                        }
                        RecordingConsentState::Rejected => {
                            self.track_controller
                                .play_track(self.settings.language, Track::RecordingStoppedReject);
                        }
                    }

                    self.recording_consent_state = RecordingConsentState::None;
                }

                self.have_active_running_recording = now_recording_active;

                self.evaluate_microphone_publish_state().await;
            }
            OpenTalkRecordingEvent::Error(..) => {
                // ignore
            }
        }

        Ok(())
    }

    fn handle_breakout_event(&mut self, breakout_event: BreakoutEvent) -> Result<()> {
        let opentalk_room = self
            .opentalk_room
            .as_mut()
            .context("Called handle_breakout_event without a room set")?;

        match &breakout_event {
            BreakoutEvent::ParticipantSwitchedRoom {
                participant_id,
                old_room,
                new_room,
                module_data: _,
            } => {
                let current_room = &opentalk_room.breakout_api().current_room();

                let participant = opentalk_room
                    .participants()
                    .get(participant_id)
                    .context("Failed to get participant state")?
                    .clone();

                if old_room == current_room {
                    for connection in participant.connections {
                        self.handle_participant_left(participant.id, connection);
                    }
                } else if new_room == current_room {
                    self.handle_participant_joined(&participant);
                }
            }
            BreakoutEvent::SwitchedRoom { .. } => {
                // Obelisk currently doesn't support joining breakout rooms, so SwitchedRoom events
                // are not expected to be received
                bail!("Got unexpected SwitchedRoom event");
            }
            BreakoutEvent::Started { .. }
            | BreakoutEvent::CloseNotice { .. }
            | BreakoutEvent::Closing { .. }
            | BreakoutEvent::Closed
            | BreakoutEvent::Error(..) => {}
        }

        Ok(())
    }

    fn handle_participant_joined(&mut self, participant: &Participant) {
        log::debug!("Participant {} joined!", participant.id);

        if let Some(compositor) = &mut self.media.compositor {
            for connection_id in &participant.connections {
                compositor.add_participant(
                    &format!("{}:{connection_id}", participant.id).into(),
                    participant.display_name.to_string(),
                );
            }
        }
    }

    fn handle_participant_left(
        &mut self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) {
        if let Some(compositor) = &mut self.media.compositor {
            compositor.remove_participant(&format!("{participant_id}:{connection_id}").into());
        }
    }

    /// Called when the id & pin are entered, checks validity and stored the connect token, which is used after the announcement is played out or skipped
    async fn start_connect_to_conference(&mut self) -> Result<()> {
        let name = self.name.as_ref().map_or("Anonymous", |s| s.as_str());

        let connect_token = self
            .opentalk_client
            .start_call_in(
                self.dtmf_id
                    .clone()
                    .parse()
                    .context("Failed to parse dtmf id to CallInId")?,
                self.dtmf_pw
                    .clone()
                    .parse()
                    .context("Failed to parse dtmf password to CallInPassword")?,
                DisplayName::from_str_lossy(name),
            )
            .await
            .context("Failed to connect to conference")?;

        self.connect_token = Some(connect_token);

        Ok(())
    }

    async fn connect_to_conference(&mut self) -> Result<()> {
        let connect_token = self
            .connect_token
            .take()
            .context("Called connect_to_conference without connect_token set")?;

        let join_result = self
            .opentalk_client
            .connect_call_in(connect_token)
            .await
            .context("Failed to connect to conference")?;

        match join_result {
            JoinResult::InRoom(room) => {
                self.opentalk_room = Some(room);
                self.handle_join_result().await?;
            }
            JoinResult::InWaitingRoom(room) => {
                self.track_controller
                    .play_track(self.settings.language, Track::WaitingRoomEntered);

                self.media.ui_controller.set_waiting_room(true);

                self.opentalk_room = Some(room);
            }
        };

        Ok(())
    }

    async fn handle_join_result(&mut self) -> Result<()> {
        let room = self
            .opentalk_room
            .as_mut()
            .context("Called handle_join_result with a current room")?;

        let credentials = room
            .livekit_credentials()
            .context("Joined room is missing the livekit module")?
            .clone();

        let mut compositor = Mixer::new(MixerParameters {
            target_fps: u16::try_from(self.media.target_fps)
                .expect("conversion to u16 not possible"),
            clock_format: self.settings.clock_format.clone(),
            livekit_url: credentials.service_url.unwrap_or(credentials.public_url),
            livekit_token: credentials.token,
            auto_subscribe: false,
        })
        .await
        .context("Failed to create Mixer")?;

        for participant in room.participants().values() {
            for connection_id in &participant.connections {
                compositor.add_participant(
                    &format!("{}:{}", participant.id, connection_id).into(),
                    participant.display_name.to_string(),
                );
            }
        }

        self.media
            .add_compositor(compositor)
            .await
            .context("Failed to add compositor to media pipeline")?;

        if is_any_recording_active(room) {
            self.track_controller
                .play_track(self.settings.language, Track::ConsentInfo);
            self.recording_consent_state = RecordingConsentState::WaitingForConsent;
        }

        let participant_id = room.participant_id();

        let livekit_api = room
            .livekit_api()
            .context("Room is missing the livekit module")?;

        let microphone_enable = match livekit_api.microphone_restriction_state() {
            MicrophoneRestrictionState::Disabled => true,
            MicrophoneRestrictionState::Enabled {
                unrestricted_participants,
            } => unrestricted_participants.contains(&participant_id),
        };

        if !microphone_enable {
            self.track_controller
                .play_track(self.settings.language, Track::MicrophonesRestrictionEnabled);
        }

        self.evaluate_microphone_publish_state().await;

        Ok(())
    }

    // Evaluate if the microphone should be published or not
    async fn evaluate_microphone_publish_state(&mut self) {
        let Some(opentalk_room) = &mut self.opentalk_room else {
            return;
        };

        let participant_id = opentalk_room.participant_id();

        let Some(livekit_api) = opentalk_room.livekit_api() else {
            return;
        };

        let mut publish = match livekit_api.microphone_restriction_state() {
            MicrophoneRestrictionState::Disabled => true,
            MicrophoneRestrictionState::Enabled {
                unrestricted_participants,
            } => unrestricted_participants.contains(&participant_id),
        };

        if let RecordingConsentState::Rejected | RecordingConsentState::WaitingForConsent =
            self.recording_consent_state
        {
            publish = false;
            self.audio_muted = true;
        }

        // The web frontend needs a mute event on the audio track to display the microphone muted symbol, just removing audio track doesn't work.
        if self.audio_muted
            && let Some(compositor) = &mut self.media.compositor
        {
            for track in compositor
                .local_participant()
                .track_publications()
                .into_values()
            {
                if track.kind() == TrackKind::Audio {
                    track.mute();
                }
            }
        }

        self.media
            .set_microphone_publish(publish, self.audio_muted)
            .await;
    }

    async fn toggle_audio_mute(&mut self) {
        // Do not toggle anything if there's nothing to toggle
        if !self.media.is_microphone_published().await {
            if matches!(
                self.recording_consent_state,
                RecordingConsentState::Rejected
            ) {
                // Play the reject message again if user tries to unmute
                self.track_controller
                    .play_track(self.settings.language, Track::ConsentReject);
            } else {
                self.track_controller
                    .play_track(self.settings.language, Track::MicrophonesRestrictionEnabled);
            }
            return;
        }

        if let Some(compositor) = &mut self.media.compositor {
            for track in compositor
                .local_participant()
                .track_publications()
                .into_values()
            {
                if track.kind() != TrackKind::Audio {
                    continue;
                }

                if self.audio_muted {
                    track.unmute();
                } else {
                    track.mute();
                }
            }
        }

        self.audio_muted = !self.audio_muted;

        self.track_controller.play_track(
            self.settings.language,
            if self.audio_muted {
                Track::On
            } else {
                Track::Off
            },
        );
    }

    fn toggle_video_mute(&mut self, play_sound: bool) {
        if self.video_muted
            && matches!(
                self.recording_consent_state,
                RecordingConsentState::Rejected
            )
        {
            // Play the reject message again if user tries to unmute
            self.track_controller
                .play_track(self.settings.language, Track::ConsentReject);
            return;
        }

        if let Some(compositor) = &mut self.media.compositor {
            for track in compositor
                .local_participant()
                .track_publications()
                .into_values()
            {
                if track.kind() != TrackKind::Video {
                    continue;
                }

                if self.video_muted {
                    track.unmute();
                } else {
                    track.mute();
                }
            }
        }

        self.video_muted = !self.video_muted;

        if play_sound {
            self.track_controller.play_track(
                self.settings.language,
                if self.video_muted {
                    Track::On
                } else {
                    Track::Off
                },
            );
        }
    }
}

fn is_any_recording_active(opentalk_room: &mut Room) -> bool {
    let Some(recording_api) = opentalk_room.recording_api() else {
        return false;
    };

    recording_api.recording_status().is_running()
        || recording_api
            .streams()
            .iter()
            .any(|(_, streaming_target)| streaming_target.status.is_running())
}

async fn handle_sip_event(
    invite_session: &mut InviteSession,
    event: InviteSessionEvent,
    media: &mut MediaPipeline,
    media_session: &mut SessionState,
) -> Result<Option<State>> {
    match event {
        InviteSessionEvent::RefreshNeeded => {
            if let Err(e) = refresh_invite_session(media, media_session, invite_session).await {
                log::error!("failed to refresh the INVITE session, {e:?}");
                return Ok(Some(State::Quitting));
            }
        }
        InviteSessionEvent::ReInviteReceived(event) => {
            if let Err(e) = handle_reinvite(media, media_session, invite_session, event).await {
                log::error!("failed to handle re-invite, {e:?}");
                return Ok(Some(State::Quitting));
            }
        }
        InviteSessionEvent::Bye(event) => {
            invite_session.handle_bye(event).await?;
            return Ok(Some(State::Terminated));
        }
        InviteSessionEvent::Terminated => {
            return Ok(Some(State::Terminated));
        }
        InviteSessionEvent::Notify(..) => {}
    }

    Ok(None)
}

async fn opentalk_room_receive(room: &mut Option<Room>) -> Result<OpenTalkEvent> {
    if let Some(room) = room {
        room.recv().await
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

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use core::{
    pin::Pin,
    task::{ready, Context, Poll},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
    io,
    sync::Arc,
};

use anyhow::{bail, Context as ErrorContext, Result};
use bytes::Bytes;
use futures::Stream;
use log::error;
use opentalk_client_signaling::{
    opentalk_roomserver_types::{
        breakout::{breakout_id::BreakoutId, event::BreakoutEvent},
        connection_id::ConnectionId,
        room_kind::RoomKind,
    },
    opentalk_roomserver_types_recording::{RecordingStatus, StreamErrorReason, StreamStatus},
    opentalk_types::{
        common::{
            rooms::RoomId, streaming::StreamingTargetId, time::Timestamp, users::DisplayName,
        },
        signaling::ParticipantId,
    },
    JoinedLobby, OpenTalkClient, OpenTalkEvent, OpenTalkRecordingServiceEvent, Participant, Room,
};
use opentalk_compositor::{
    EncoderType, Mixer, MixerParameters, ParticipantIdentity, RTMPParameters, RTMPSink, SystemSink,
    WebMParameters, WebMSink,
};
use opentalk_orchestrator_client::{client::OrchestratorHandle, RecorderEvent, RecorderResource};
use opentalk_types_api_internal::recording::RecordingTarget;
use reqwest::Url;
use thiserror::Error;
use tokio::{
    io::{AsyncRead, ReadBuf},
    sync::{broadcast, watch},
};
use tokio_stream::wrappers::BroadcastStream;

use crate::settings::Settings;

const DISPLAY_NAME: &str = "recorder";

#[derive(Debug, Error)]
pub(crate) enum RecordingSessionError {
    #[error("Stream '{0}' is already running")]
    AlreadyRunning(StreamingTargetId),
    #[error("Stream '{0}' not found")]
    NotFound(StreamingTargetId),
    #[error("Stream '{0}' is not running")]
    NotRunning(StreamingTargetId),

    #[error("Recording is not running")]
    NoRunningRecording,

    #[error("Start livestream failed")]
    FailedToStartLivestream(#[source] anyhow::Error),

    #[error("Start recording failed")]
    FailedToStartRecording(#[source] anyhow::Error),
}

impl From<RecordingSessionError> for StreamErrorReason {
    fn from(value: RecordingSessionError) -> Self {
        let code = match value {
            RecordingSessionError::AlreadyRunning(_) => "already_running".to_owned(),
            RecordingSessionError::NotFound(_) => "not_found".to_owned(),
            RecordingSessionError::NotRunning(_) => "not_running".to_owned(),
            RecordingSessionError::NoRunningRecording => "no_running_recording".to_owned(),
            RecordingSessionError::FailedToStartLivestream(_) => "start_livestream".to_owned(),
            RecordingSessionError::FailedToStartRecording(_) => "start_recording".to_owned(),
        };

        Self {
            code,
            message: format!("{value:?}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UploadLimitReached(Option<StreamingTargetId>);

// TODO: Replace with version from opentalk-types
#[derive(Clone)]
pub(crate) struct FileExtension(String);

impl FileExtension {
    #[must_use]
    pub(crate) fn webm() -> Self {
        Self("webm".to_string())
    }

    #[must_use]
    pub(crate) fn str(&self) -> &str {
        &self.0
    }
}

pub(crate) struct Recorder {
    pub(crate) settings: Arc<Settings>,
    pub(crate) client: Arc<OpenTalkClient>,
    pub(crate) shutdown: broadcast::Receiver<()>,
}

impl Clone for Recorder {
    fn clone(&self) -> Self {
        Self {
            settings: self.settings.clone(),
            client: self.client.clone(),
            shutdown: self.shutdown.resubscribe(),
        }
    }
}

impl Recorder {
    /// This constructor is used by the integration tests to mock data.
    pub(crate) fn new(
        settings: Arc<Settings>,
        client: OpenTalkClient,
        shutdown: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            settings,
            client: Arc::new(client),
            shutdown,
        }
    }

    pub(crate) async fn spawn_session(
        &self,
        recording_target: RecordingTarget,
        orchestrator_handle: Option<OrchestratorHandle>,
    ) -> Result<RecordingSessionHandle> {
        let context = Arc::new(self.clone());
        log::debug!("Start Recording session {recording_target:?}");
        let (mut session, handle) =
            match Box::pin(RecordingSession::create(context, recording_target)).await {
                Ok(session) => session,
                Err(err) => {
                    notify_orchestrator_recording_stopped(orchestrator_handle, recording_target)
                        .await;
                    return Err(err.context("recording session failed to start"));
                }
            };

        tokio::spawn(async move {
            if let Err(ref recording_err) = Box::pin(session.run()).await {
                error!("recording session failed but trying upload anyway:\n{recording_err:?}");
            }

            notify_orchestrator_recording_stopped(orchestrator_handle, recording_target).await;
        });

        Ok(handle)
    }
}

#[derive(Clone)]
struct LivestreamConfigurationAndStatus {
    location: Url,
    status: StreamStatus,
}

async fn notify_orchestrator_recording_stopped(
    orchestrator_handle: Option<OrchestratorHandle>,
    recording_target: RecordingTarget,
) {
    if let Some(handle) = orchestrator_handle {
        if let Err(e) = handle
            .send_event(RecorderEvent::RemoveRecording(RecorderResource {
                room_id: recording_target.room_id,
                breakout_id: recording_target.breakout_room,
            }))
            .await
        {
            error!("Failed to send RemoveRoom event to orchestrator: {e}");
        }
    }
}

#[derive(Debug)]
pub enum SessionState {
    Run,
    Exit,
}

/// The handle for a [`RecordingSession`]
#[derive(Debug, Clone)]
pub(crate) struct RecordingSessionHandle {
    state_sender: watch::Sender<SessionState>,
}

impl RecordingSessionHandle {
    /// Signal the transcription session to exit
    pub(crate) fn exit_task(&self) {
        // can only fail if the receiver is dropped -> 'successful' exit
        let _ = self.state_sender.send(SessionState::Exit);
    }

    /// Check if the associated session is finished
    pub(crate) fn is_finished(&self) -> bool {
        self.state_sender.is_closed()
    }
}

pub(crate) struct RecordingSession {
    service_context: Arc<Recorder>,
    state_receiver: watch::Receiver<SessionState>,
    room_state: opentalk_client_signaling::Room,
    room_id: RoomId,

    compositor: Mixer,

    recording_status: RecordingStatus,
    livestream_states: BTreeMap<StreamingTargetId, LivestreamConfigurationAndStatus>,

    done: bool,
}

impl RecordingSession {
    /// Start a new recording session at the specified target room
    pub(crate) async fn create(
        service_context: Arc<Recorder>,
        recording_target: RecordingTarget,
    ) -> Result<(RecordingSession, RecordingSessionHandle)> {
        // Connect to opentalk room as recorder
        let JoinedLobby {
            room: mut room_state,
            can_enter,
            display_name,
        } = Box::pin(service_context.client.connect_recorder(
            recording_target.room_id,
            recording_target.breakout_room.map(BreakoutId::from),
        ))
        .await?;

        if !can_enter {
            bail!("not allowed to enter the room");
        }

        let display_name = display_name
            .is_none()
            .then(|| DisplayName::from_str_lossy(DISPLAY_NAME));
        room_state.enter_room(display_name).await?;

        if let Some(breakout_target) = recording_target.breakout_room {
            move_to_breakout_room(&mut room_state, BreakoutId::from(breakout_target)).await?;
        }

        // Extract recording state from room
        let recording_api = room_state
            .recording_api()
            .context("recording_api not available")?;

        let recording_status = recording_api.recording_status().clone();

        let stream_states = recording_api.streams().clone();
        let livestream_states = room_state
            .recording_service_api()
            .context("recording_service_api not available")?
            .streaming_targets()
            .iter()
            .map(|(id, target)| -> Result<_> {
                Ok((
                    *id,
                    LivestreamConfigurationAndStatus {
                        location: target.location.clone(),
                        status: stream_states
                            .get(id)
                            .context("Missing stream state for streaming target")?
                            .status
                            .clone(),
                    },
                ))
            })
            .collect::<Result<_>>()?;

        // Create Compositor
        let livekit_credentials = room_state
            .livekit_credentials()
            .cloned()
            .context("Missing livekit credentials in room")?;

        let recorder_settings = service_context
            .settings
            .recorder
            .clone()
            .unwrap_or_default();

        let compositor_params = MixerParameters {
            target_fps: 30,
            auto_subscribe: false,
            clock_format: recorder_settings.clock_format,
            livekit_url: livekit_credentials
                .service_url
                .as_ref()
                .unwrap_or(&livekit_credentials.public_url)
                .clone(),
            livekit_token: livekit_credentials.token.clone(),
        };

        let mut compositor = Box::pin(Mixer::new(compositor_params))
            .await
            .context("Failed to create compositor")?;

        // Add display sink if enabled
        if recorder_settings.display {
            let system_sink = SystemSink::create().context("DisplaySink could not created")?;
            compositor
                .link_gstreamer_sink("Display", system_sink)
                .await?;
        }

        let (state_sender, state_receiver) = watch::channel(SessionState::Run);

        Ok((
            RecordingSession {
                service_context,
                state_receiver,
                room_state,
                room_id: recording_target.room_id,
                compositor,
                recording_status,
                livestream_states,
                done: false,
            },
            RecordingSessionHandle { state_sender },
        ))
    }

    pub(crate) async fn run(&mut self) -> Result<()> {
        let mut shutdown_rx = self.service_context.shutdown.resubscribe();
        let (chunk_limit_reached_tx, mut chunk_limit_reached_rx) =
            broadcast::channel::<UploadLimitReached>(1);

        self.initialize(chunk_limit_reached_tx.clone()).await?;
        let mut state_receiver = self.state_receiver.clone();

        while !self.done {
            tokio::select! {
                msg = self.room_state.recv() => {
                    match msg {
                        Err(err) => {
                            log::debug!("Failed to receive next websocket message: {err:?}");
                            break;
                        },
                        Ok(event) => if let Err(err) = Box::pin(self.handle_signaling_event(event, chunk_limit_reached_tx.clone())).await {
                            log::error!("Failed to handle signaling event: {err:?}");
                            break;
                        },
                    }
                }
                disconnect_reason = self.compositor.run() => {
                    log::error!("Disconnected from livekit: {disconnect_reason:?}");
                    break;
                }
                chunk_limit_event = chunk_limit_reached_rx.recv() => {
                    if let Some(streaming_target_id) = chunk_limit_event.context("Lost chunk limit receiver")?.0 {
                        if let Err(e) = self.handle_stop_stream(BTreeSet::from([streaming_target_id])).await {
                            log::error!("Error while stopping stream {e}");
                            break;
                        }
                    } else if let Err(e) = self.handle_stop_recording().await {
                        log::error!("Error while stopping recording: {e}");
                        break;
                    }
                }
                res = state_receiver.changed() => {
                    if matches!(*state_receiver.borrow_and_update(), SessionState::Exit) || res.is_err() {
                        break;
                    }
                }
                _ = shutdown_rx.recv() => {
                    break;
                }
            }
        }

        // The streaming targets are per session
        // therefore making sure we're in the right context isn't necessary here.
        log::debug!("Recorder is done, attempting to upload remaining streams...");
        if self.recording_status.is_running() {
            if let Err(err) = self.handle_stop_recording().await {
                log::error!("Failed to stop recording: {err:?}");
            };
        }

        let running_stream_ids = self
            .livestream_states
            .iter()
            .filter(|(_, livestream)| livestream.status.is_running())
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        for stream_target_id in running_stream_ids {
            if let Err(err) = self.stop_stream(stream_target_id).await {
                log::error!("Failed to stop stream {stream_target_id}: {err:?}");
            }
        }

        self.compositor.close().await;

        Ok(())
    }

    pub(crate) async fn start_recording(
        &mut self,
        sender: broadcast::Sender<UploadLimitReached>,
    ) -> Result<RecordingStatus> {
        if self.recording_status.is_running() {
            log::debug!("Received start recording while already recording, ignoring");
            return Ok(self.recording_status.clone());
        }

        let websocket_base_url = self
            .service_context
            .settings
            .controller
            .url
            .clone()
            .to_websocket_url();

        let mut upload_url = websocket_base_url
            .join("/internal/recording/upload")
            .context("Failed to join controller url with upload path")?;

        upload_url.set_query(Some(&format!(
            "room_id={room_id}&file_extension={file_extension}&timestamp={timestamp}",
            room_id = self.room_id,
            file_extension = FileExtension::webm().str(),
            timestamp = urlencoding::encode(&Timestamp::now().to_string()),
        )));

        let webm_sink = WebMSink::create(&WebMParameters {
            encoder_type: self.service_context.settings.encoder_type(),
            chunk_size: Some(self.service_context.settings.controller.upload_chunk_size as u64),
        })
        .context("WebM-Sink could not created")
        .map_err(RecordingSessionError::FailedToStartRecording)?;

        let receiver = webm_sink.subscribe();

        self.compositor
            .link_gstreamer_sink("recording", webm_sink)
            .await
            .context("unable to link recording sink to compositor")?;

        // probably actually use a channel to signal when limit reached, would probably make much more sense
        // than to attempt to circumvent the move
        tokio::spawn({
            let service_context = self.service_context.clone();

            async move {
                let stream = BroadcastStream::from(receiver);

                match service_context
                    .client
                    .upload_render(upload_url.as_str(), stream, || {
                        if let Err(e) = sender.send(UploadLimitReached(None)) {
                            log::warn!(
                                "Could not send Upload limit reached of recording because: {e}"
                            );
                        }
                    })
                    .await
                {
                    Ok(()) => {}
                    Err(err) => {
                        log::error!(
                            "Encountered error during recording upload, {:?}",
                            anyhow::Error::from(err)
                        );
                    }
                }
            }
        });

        Ok(RecordingStatus::Active)
    }

    async fn start_livestream(
        &mut self,
        id: StreamingTargetId,
    ) -> Result<StreamStatus, RecordingSessionError> {
        log::trace!("start_stream, id: {id:?}");
        let Some(livestream_state) = self.livestream_states.get_mut(&id) else {
            return Err(RecordingSessionError::NotFound(id));
        };

        if livestream_state.status == StreamStatus::Active {
            return Err(RecordingSessionError::AlreadyRunning(id));
        }

        let result = Self::setup_livestream_in_mixer(
            &mut self.compositor,
            format!("Livestream-{id}"),
            &livestream_state.location,
            self.service_context.settings.encoder_type(),
        )
        .await;

        if let Err(ref e) = result {
            log::error!("failed to start live stream: {e}");
        }

        livestream_state.status =
            match result.map_err(RecordingSessionError::FailedToStartLivestream) {
                Ok(()) => StreamStatus::Active,
                Err(error) => StreamStatus::Error {
                    reason: error.into(),
                },
            };

        Ok(livestream_state.status.clone())
    }

    async fn setup_livestream_in_mixer(
        compositor: &mut Mixer,
        name: String,
        location: &Url,
        encoder_type: EncoderType,
    ) -> Result<()> {
        let rtmp_sink = RTMPSink::create(RTMPParameters {
            location: location.to_string(),
            audio_bitrate: None,
            audio_rate: None,
            video_bitrate: None,
            video_speed_preset: None,
            encoder_type,
        })
        .context("Failed to create RTMP sink from livestream configuration")?;

        compositor
            .link_gstreamer_sink(&name, rtmp_sink)
            .await
            .context("unable to link sink to compositor")
    }

    async fn stop_stream(
        &mut self,
        id: StreamingTargetId,
    ) -> Result<StreamStatus, RecordingSessionError> {
        log::trace!("stop_stream, id: {id:?}");
        let Some(livestream_state) = self.livestream_states.get_mut(&id) else {
            return Err(RecordingSessionError::NotFound(id));
        };

        if !livestream_state.status.is_running() {
            return Err(RecordingSessionError::NotRunning(id));
        }

        self.compositor
            .release_sink(&format!("Livestream-{id}"))
            .await;

        livestream_state.status = StreamStatus::Inactive;

        Ok(StreamStatus::Inactive)
    }

    async fn stop_recording(&mut self) -> Result<RecordingStatus, RecordingSessionError> {
        if !self.recording_status.is_running() {
            return Err(RecordingSessionError::NoRunningRecording);
        }

        self.compositor.release_sink(&"recording".to_owned()).await;

        Ok(RecordingStatus::Inactive)
    }

    async fn handle_signaling_event(
        &mut self,
        event: OpenTalkEvent,
        chunk_limit_sender: broadcast::Sender<UploadLimitReached>,
    ) -> Result<()> {
        log::trace!("Received: {event:?} event message");
        match event {
            OpenTalkEvent::ParticipantJoined(participant) => {
                self.handle_participant_joined(&participant);
            }
            OpenTalkEvent::ParticipantUpdated {
                previous: _,
                updated,
            } => {
                self.handle_participant_updated(&updated);
            }
            OpenTalkEvent::ParticipantLeft(left, connection) => {
                self.handle_participant_left(&left, connection);
            }
            OpenTalkEvent::Breakout(breakout_event) => {
                self.handle_breakout_event(breakout_event)?;
            }
            OpenTalkEvent::EntryPermissionChanged { can_enter: _ }
            | OpenTalkEvent::DisplayNameAssigned(_)
            | OpenTalkEvent::WaitingRoomAccepted
            | OpenTalkEvent::LiveKit(_)
            | OpenTalkEvent::Recording(_)
            | OpenTalkEvent::Transcription(_)
            | OpenTalkEvent::TranscriptionService(_) => (),
            OpenTalkEvent::MovedToWaitingRoom => {
                log::error!("Recorder was moved to waiting room, stopping recording");
                self.done = true;
            }
            OpenTalkEvent::Disconnected(reason) => {
                log::info!("Disconnected from roomserver signaling: {reason:?}");
                self.done = true;
            }
            OpenTalkEvent::RecordingService(open_talk_recording_service_event) => {
                match open_talk_recording_service_event {
                    OpenTalkRecordingServiceEvent::StartRecording => {
                        log::debug!("Start recording");
                        self.handle_start_recording(chunk_limit_sender).await?;
                    }
                    OpenTalkRecordingServiceEvent::PauseRecording => {
                        log::debug!("Pause recording (not implemented)");
                    }
                    OpenTalkRecordingServiceEvent::StopRecording => {
                        log::debug!("Stop recording");
                        self.handle_stop_recording().await?;
                    }
                    OpenTalkRecordingServiceEvent::StartStreams { target_ids } => {
                        log::debug!("Start streams: {target_ids:?}");
                        self.handle_start_stream(target_ids).await?;
                    }
                    OpenTalkRecordingServiceEvent::PauseStreams { target_ids } => {
                        log::debug!("Pause streams (not implemented): {target_ids:?}");
                    }
                    OpenTalkRecordingServiceEvent::StopStreams { target_ids } => {
                        log::debug!("Stop streams: {target_ids:?}");
                        self.handle_stop_stream(target_ids).await?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Initialize the recording session
    ///
    /// Called once at the very beginning after connecting to the conference
    pub(crate) async fn initialize(
        &mut self,
        chunk_limit_sender: broadcast::Sender<UploadLimitReached>,
    ) -> Result<()> {
        // Test if there's anything to do & bail early if not
        let recording_inactive = self.recording_status == RecordingStatus::Inactive;
        let livestreams_inactive = self
            .livestream_states
            .iter()
            .all(|(_, rec_info)| rec_info.status == StreamStatus::Inactive);

        if recording_inactive && livestreams_inactive {
            log::debug!("No streams to start requested, recorder is done.");
            self.done = true;
            return Ok(());
        }

        // Start requested livestreams & recordings
        if self.recording_status == RecordingStatus::Requested {
            self.handle_start_recording(chunk_limit_sender.clone())
                .await?;
        }

        let all_streams = {
            let recording_api = self
                .room_state
                .recording_api()
                .context("recording_api unavailable")?;
            recording_api.streams().clone()
        };

        for (id, _status) in all_streams
            .into_iter()
            .filter(|(_id, state)| matches!(state.status, StreamStatus::Requested))
        {
            let status = match self.start_livestream(id).await {
                Ok(status) => status,
                Err(reason) => StreamStatus::Error {
                    reason: reason.into(),
                },
            };

            self.room_state
                .recording_service_api()
                .context("recording_service_api unavailable")?
                .send_stream_update(id, status)
                .await?;
        }

        if let Some(event_title) = self.room_state.event_title() {
            self.compositor.set_event_title(event_title);
        }

        for participant in self.room_state.participants().values() {
            if participant.consents_recording {
                for connection in &participant.connections {
                    self.compositor.add_participant(
                        &format!("{}:{}", participant.id, connection).into(),
                        participant.display_name.to_string(),
                    );
                }
            }
        }

        Ok(())
    }

    fn handle_participant_joined(&mut self, participant: &Participant) {
        if participant.consents_recording {
            for connection in &participant.connections {
                self.compositor.add_participant(
                    &ParticipantIdentity::from(format!("{}:{}", participant.id, connection)),
                    participant.display_name.to_string(),
                );
            }
        }
    }

    fn handle_participant_updated(&mut self, updated: &Participant) {
        if updated.consents_recording {
            for connection in &updated.connections {
                self.compositor.add_participant(
                    &ParticipantIdentity::from(format!("{}:{}", updated.id, connection)),
                    updated.display_name.to_string(),
                );
            }
        } else {
            for connection in &updated.connections {
                self.compositor
                    .remove_participant(&ParticipantIdentity::from(format!(
                        "{}:{}",
                        updated.id, connection
                    )));
            }
        }
    }

    fn handle_participant_left(&mut self, left: &ParticipantId, connection: ConnectionId) {
        let active_participants = self.room_state.active_participants();
        if active_participants.is_empty() {
            log::debug!("Last participant left the session. Stop recording.");
            self.done = true;

            return;
        }

        log::trace!(
            "{} remaining participants : {:?}",
            active_participants.len(),
            active_participants.keys()
        );

        self.compositor
            .remove_participant(&ParticipantIdentity::from(format!("{left}:{connection}")));
    }

    fn handle_breakout_event(&mut self, breakout_event: BreakoutEvent) -> Result<()> {
        match breakout_event {
            BreakoutEvent::ParticipantSwitchedRoom {
                participant_id,
                old_room,
                new_room,
                module_data: _,
            } => {
                let current_room = self.room_state.breakout_api().current_room();

                let participant = self
                    .room_state
                    .participants()
                    .get(&participant_id)
                    .context("Failed to get participant state")?
                    .clone();

                if old_room == current_room {
                    // remove participant from recording
                    for connection in participant.connections {
                        self.handle_participant_left(&participant.id, connection);
                    }
                } else if new_room == current_room {
                    // add participant to recording if consenting
                    self.handle_participant_joined(&participant);
                }
            }
            BreakoutEvent::Closed => {
                log::info!("Breakout room was closed, stopping recording");
                self.done = true;
            }
            BreakoutEvent::Error(breakout_error) => {
                log::error!("Received unexpected error from breakout module: {breakout_error:?}");
            }

            BreakoutEvent::Started { .. }
            | BreakoutEvent::SwitchedRoom { .. }
            | BreakoutEvent::CloseNotice { .. }
            | BreakoutEvent::Closing { .. } => (),
        }

        Ok(())
    }

    pub(crate) async fn handle_start_recording(
        &mut self,
        chunk_limit_sender: broadcast::Sender<UploadLimitReached>,
    ) -> Result<()> {
        let status = match self.start_recording(chunk_limit_sender.clone()).await {
            Ok(status) => status,
            Err(reason) => RecordingStatus::Error {
                reason: RecordingSessionError::FailedToStartRecording(reason).into(),
            },
        };

        self.recording_status = status.clone();

        self.room_state
            .recording_service_api()
            .context("recording_service_api unavailable")?
            .send_recording_update(status)
            .await?;

        Ok(())
    }

    pub(crate) async fn handle_stop_recording(&mut self) -> Result<()> {
        let status = match self.stop_recording().await {
            Ok(status) => status,
            Err(reason) => RecordingStatus::Error {
                reason: reason.into(),
            },
        };

        self.recording_status = status.clone();

        self.room_state
            .recording_service_api()
            .context("recording_service_api unavailable")?
            .send_recording_update(status)
            .await?;

        self.evaluate_if_done();

        Ok(())
    }

    pub(crate) async fn handle_start_stream(
        &mut self,
        target_ids: BTreeSet<StreamingTargetId>,
    ) -> Result<()> {
        for id in target_ids {
            let status = match self.start_livestream(id).await {
                Ok(status) => status,
                Err(reason) => StreamStatus::Error {
                    reason: reason.into(),
                },
            };

            self.room_state
                .recording_service_api()
                .context("recording_service_api unavailable")?
                .send_stream_update(id, status)
                .await?;
        }

        Ok(())
    }

    async fn handle_stop_stream(&mut self, target_ids: BTreeSet<StreamingTargetId>) -> Result<()> {
        for id in target_ids {
            let status = match self.stop_stream(id).await {
                Ok(status) => status,
                Err(reason) => StreamStatus::Error {
                    reason: reason.into(),
                },
            };

            self.room_state
                .recording_service_api()
                .context("recording_service_api unavailable")?
                .send_stream_update(id, status)
                .await?;
        }

        self.evaluate_if_done();

        Ok(())
    }

    /// Check if there is anything left to do, quit if not
    fn evaluate_if_done(&mut self) {
        if !self.recording_status.is_running()
            && !self
                .livestream_states
                .iter()
                .any(|(_id, livestream_state)| livestream_state.status.is_running())
        {
            // last stream has been stopped, the media pipeline can be shut down.
            self.done = true;
        }
    }
}

async fn move_to_breakout_room(room_state: &mut Room, breakout_target: BreakoutId) -> Result<()> {
    log::info!("Moving to breakout room {breakout_target}");
    let Some(breakout_config) = room_state.breakout_api().breakout_config() else {
        return Err(anyhow::anyhow!(
            "Received breakout target {breakout_target} but there are no breakout rooms configured"
        ));
    };

    if !breakout_config
        .rooms
        .iter()
        .any(|room| room.id == breakout_target)
    {
        return Err(anyhow::anyhow!("Got breakout target {breakout_target} but it was not found in the room's breakout configuration"));
    }

    room_state
        .breakout_api()
        .switch_room(RoomKind::Breakout(breakout_target))
        .await?;

    log::info!("Entering breakout room, waiting for breakout switch event...");
    loop {
        match room_state.recv().await? {
            OpenTalkEvent::MovedToWaitingRoom => {
                bail!("Recorder was moved to the waiting room, cannot start recording")
            }
            OpenTalkEvent::Breakout(breakout_event) => match breakout_event {
                BreakoutEvent::SwitchedRoom { .. } => {
                    log::info!("Received breakout switched room event, recording can be started");
                    return Ok(());
                }
                BreakoutEvent::Closing { .. } | BreakoutEvent::Closed => {
                    bail!("Targeted breakout room was closed, cannot start recording")
                }
                BreakoutEvent::Error(breakout_error) => {
                    bail!("Error in breakout module: {breakout_error:?}")
                }
                _ => (),
            },
            OpenTalkEvent::Disconnected(disconnect_reason) => {
                bail!("Disconnected from room with reason: {disconnect_reason:?}")
            }
            event => log::trace!("Received event while waiting for breakout switch: {event:?}"),
        }
    }
}

pin_project_lite::pin_project! {
    /// Helper struct which reads an opened file and returns chunks of up to 8kb as Stream
    struct FileReadStream {
        #[pin]
        file: tokio::fs::File,
    }
}

impl Stream for FileReadStream {
    type Item = Result<Bytes, io::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let mut buf = [0u8; 8192];
        let mut read_buf = ReadBuf::new(&mut buf);
        ready!(this.file.poll_read(cx, &mut read_buf))?;

        let buffer = read_buf.filled();
        if buffer.is_empty() {
            return Poll::Ready(None);
        }

        Poll::Ready(Some(Ok(Bytes::copy_from_slice(buffer))))
    }
}

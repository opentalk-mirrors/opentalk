// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, hash_map::Entry};

use anyhow::Context;
use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    participant_state::ParticipantState,
    signaling_module::{
        ModuleJoinData, ModuleSwitchData, NoOp, PeerDataMap, SignalingModule,
        SignalingModuleDescription, SignalingModuleFeatureDescription, SignalingModuleInitData,
    },
};
use opentalk_roomserver_types::{
    breakout::BreakoutRoom,
    client_parameters::ClientKind,
    connection_id::ConnectionId,
    room_kind::RoomKind,
    signaling::module_error::{FatalError, SignalingModuleError},
};
use opentalk_roomserver_types_recording::{
    RECORD_FEATURE_ID, RECORDING_MODULE_ID, RecordingSettings, RecordingStatus, STREAM_FEATURE_ID,
    StreamStatus, StreamingTarget,
    command::RecordingCommand,
    event::{RecordingError, RecordingEvent},
    peer_state::RecordingPeerState,
    service::{
        command::RecordingServiceCommand,
        event::RecordingServiceEvent,
        state::{RecordingServiceState, ServiceStreamingTarget},
    },
    state::RecordingState,
};
use opentalk_types_api_internal::recording::RecordingTarget;
use opentalk_types_common::{
    features::ModuleFeatureId,
    modules::ModuleId,
    streaming::{RoomStreamingTarget, StreamingTargetId},
};
use opentalk_types_signaling::ParticipantId;

pub struct RecordingModule {
    settings: RecordingSettings,
    http_client: reqwest::Client,

    // Features
    can_record: bool,
    can_stream: bool,

    /// Streaming targets configured for the room
    streaming_targets: Vec<RoomStreamingTarget>,

    /// Set of participants which consented to being recorded
    consenting_participants: HashSet<ParticipantId>,

    /// Recording state for each room. No entry equals `StreamStatus::Inactive`
    recording_states: HashMap<RoomKind, RecordingStatus>,

    /// Stream states and the room in which the configuration is used.
    /// No entry means the streaming target is unused
    stream_states: HashMap<StreamingTargetId, (RoomKind, StreamStatus)>,
}

pub enum LoopBackEvent {
    RecorderRequestFailed,
}

impl SignalingModuleDescription for RecordingModule {
    const MODULE_ID: ModuleId = RECORDING_MODULE_ID;
    const DESCRIPTION: &'static str = "Handles recording functionality.";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[
        SignalingModuleFeatureDescription {
            feature_id: RECORD_FEATURE_ID,
            description: "Allows creation of recordings for meetings",
        },
        SignalingModuleFeatureDescription {
            feature_id: STREAM_FEATURE_ID,
            description: "Allows streaming meetings to streaming services",
        },
    ];

    type Settings = RecordingSettings;
}

impl SignalingModule for RecordingModule {
    const NAMESPACE: ModuleId = RECORDING_MODULE_ID;

    type Incoming = RecordingCommand;
    type Outgoing = RecordingEvent;
    type Internal = NoOp;
    type Loopback = LoopBackEvent;
    type JoinInfo = RecordingState;
    type PeerJoinInfo = RecordingPeerState;
    type Error = RecordingError;

    fn init(init_data: SignalingModuleInitData) -> Option<Self> {
        if init_data.room_parameters.e2e_encryption {
            return None;
        }

        let settings = (init_data
            .room_parameters
            .module_settings
            .get::<RecordingSettings>()
            .ok()?)?;

        let tariff = &init_data.room_parameters.tariff;
        let can_record = !tariff.disabled_features.contains(&ModuleFeatureId {
            module: RECORDING_MODULE_ID,
            feature: RECORD_FEATURE_ID,
        });
        let can_stream = !tariff.disabled_features.contains(&ModuleFeatureId {
            module: RECORDING_MODULE_ID,
            feature: STREAM_FEATURE_ID,
        });

        Some(Self {
            settings,
            http_client: reqwest::Client::new(),
            can_record,
            can_stream,
            streaming_targets: init_data.room_parameters.streaming_targets.clone(),
            consenting_participants: HashSet::new(),
            recording_states: HashMap::new(),
            stream_states: HashMap::new(),
        })
    }

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        p_joined: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        let recording_state = self.build_module_state(ctx, p_joined)?;

        let mut peer_data = PeerDataMap::default();

        // Use insert_for_matching here to avoid serializing the same message multiple times
        peer_data.insert_for_matching(
            ctx,
            RecordingPeerState {
                consents_recording: true,
            },
            |participant_id, _| self.consenting_participants.contains(&participant_id),
        )?;
        peer_data.insert_for_matching(
            ctx,
            RecordingPeerState {
                consents_recording: false,
            },
            |participant_id, _| !self.consenting_participants.contains(&participant_id),
        )?;

        let mut peer_events = PeerDataMap::default();
        peer_events.insert_for_all(
            ctx,
            RecordingPeerState {
                consents_recording: self.consenting_participants.contains(&p_joined),
            },
        )?;

        Ok(ModuleJoinData {
            join_success: Some(recording_state),
            peer_events,
            peer_data,
        })
    }

    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        let participant_state = ctx
            .participant_state(participant_id)
            .context("Failed to find disconnected participant state")
            .map_err(FatalError)?;

        if participant_state.kind == (ClientKind::Recorder { room: ctx.room }) {
            self.reset_states_if_no_recorder_is_connected(ctx)
        } else {
            Ok(())
        }
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
        command: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match command {
            RecordingCommand::SetConsent { consent } => {
                self.set_consent(ctx, participant_id, consent)
            }
            RecordingCommand::StartRecording => self.start_recording(ctx, participant_id),
            RecordingCommand::PauseRecording => self.pause_recording(ctx, participant_id),
            RecordingCommand::StopRecording => self.stop_recording(ctx, participant_id),
            RecordingCommand::StartStream { target_ids } => {
                self.start_stream(ctx, participant_id, target_ids)
            }
            RecordingCommand::PauseStream { target_ids } => {
                self.pause_stream(ctx, participant_id, target_ids)
            }
            RecordingCommand::StopStream { target_ids } => {
                self.stop_stream(ctx, participant_id, target_ids)
            }
            RecordingCommand::Service { command } => {
                self.handle_service_message(ctx, participant_id, command)
            }
        }
    }

    fn on_breakout_start(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
        _rooms: &[BreakoutRoom],
        _duration: Option<std::time::Duration>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }

    fn on_breakout_switch(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _old_room: RoomKind,
        _new_room: RoomKind,
    ) -> Result<ModuleSwitchData<Self>, SignalingModuleError<Self::Error>> {
        let recording_state = self.build_module_state(ctx, participant_id)?;

        let connections = ctx
            .participant_state(participant_id)
            .with_context(|| format!("Participant '{participant_id}' does not have state"))?
            .connections();

        let switch_success = connections
            .map(|connection_id| (connection_id, Some(recording_state.clone())))
            .collect();

        Ok(ModuleSwitchData {
            switch_success,
            ..Default::default()
        })
    }

    fn on_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match event {
            LoopBackEvent::RecorderRequestFailed => {
                // Roll back requested targets since recorder could not be found
                self.reset_states_if_no_recorder_is_connected(ctx)?;

                Err(RecordingError::FailedToRequestRecordingService.into())
            }
        }
    }

    fn on_breakout_closed(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }
}

impl RecordingModule {
    fn build_module_state(
        &mut self,
        ctx: &mut ModuleContext<'_, RecordingModule>,
        participant_id: ParticipantId,
    ) -> Result<RecordingState, SignalingModuleError<RecordingError>> {
        let mut stream_states = BTreeMap::new();

        for target in &self.streaming_targets {
            let status = self
                .stream_states
                .get(&target.id)
                .map(|(room, status)| {
                    if *room == ctx.room {
                        status.clone()
                    } else {
                        StreamStatus::InUse
                    }
                })
                .unwrap_or(StreamStatus::Inactive);

            let public_url = match &target.streaming_target.kind {
                opentalk_types_common::streaming::StreamingTargetKind::Custom {
                    public_url,
                    ..
                } => public_url.clone(),
            };

            stream_states.insert(
                target.id,
                StreamingTarget {
                    name: target.streaming_target.name.clone(),
                    public_url,
                    status,
                },
            );
        }

        let mut recording_state = RecordingState {
            consents_recording: self.consenting_participants.contains(&participant_id),
            recording_state: self
                .recording_states
                .get(&ctx.room)
                .cloned()
                .unwrap_or(RecordingStatus::Inactive),
            stream_states,
            service: None,
        };

        let joined_participant_state = ctx
            .participant_state(participant_id)
            .context("Failed to find state for joined participant")
            .map_err(FatalError)?;

        if joined_participant_state.kind == (ClientKind::Recorder { room: ctx.room }) {
            let mut streaming_targets = BTreeMap::new();

            for target in &self.streaming_targets {
                let Some(location) = target.streaming_target.kind.get_stream_target_location()
                else {
                    tracing::warn!("Failed to build streaming url for a streaming-target");
                    continue;
                };

                streaming_targets.insert(target.id, ServiceStreamingTarget { location });
            }

            recording_state.service = Some(RecordingServiceState { streaming_targets });
        }

        Ok(recording_state)
    }

    fn set_consent(
        &mut self,
        ctx: &mut ModuleContext<'_, RecordingModule>,
        participant_id: ParticipantId,
        consent: bool,
    ) -> Result<(), SignalingModuleError<RecordingError>> {
        let updated = if consent {
            self.consenting_participants.insert(participant_id)
        } else {
            self.consenting_participants.remove(&participant_id)
        };

        if updated {
            ctx.send_ws_message(
                ctx.participants.connected().ids(),
                RecordingEvent::ConsentUpdated {
                    participant: participant_id,
                    consents: consent,
                },
            )?;
        }

        Ok(())
    }

    fn request_recorder(
        &mut self,
        ctx: &mut ModuleContext<'_, RecordingModule>,
    ) -> Result<(), SignalingModuleError<RecordingError>> {
        let jwt_bearer_token = self
            .settings
            .api_key
            .generate_jwt()
            .context("Failed to generate JWT from recorder api_key")
            .map_err(FatalError)?;

        let http_client = self.http_client.clone();

        let body = RecordingTarget {
            room_id: ctx.room_id,
            breakout_room: match ctx.room {
                RoomKind::Main => None,
                RoomKind::Breakout(breakout_id) => Some(breakout_id.into()),
            },
        };

        let url = match self.settings.url.join("v1/init") {
            Ok(url) => url,
            Err(err) => {
                tracing::error!(
                    "Failed to build recorder init url from base_url: {}, {err}",
                    self.settings.url
                );
                self.reset_states_if_no_recorder_is_connected(ctx)?;

                return Err(RecordingError::FailedToRequestRecordingService.into());
            }
        };
        let trace_headers = opentalk_tracing::propagation::current_context_headers();

        ctx.spawn_optional(async move {
            tracing::debug!("Sending off recorder start request to {url} with body {body:?}");

            let response = http_client
                .post(url)
                .bearer_auth(jwt_bearer_token)
                .headers(trace_headers)
                .json(&body)
                .send()
                .await;

            let response = match response {
                Ok(response) => response,
                Err(err) => {
                    tracing::error!("Failed to send init request to recorder, {err:?}");
                    return Some(LoopBackEvent::RecorderRequestFailed);
                }
            };

            let response_status = response.status();
            let response_body = response
                .bytes()
                .await
                .map(|bytes| String::from_utf8_lossy(&bytes).into_owned());

            if response_status.is_success() {
                tracing::debug!(
                    "Got recorder start response status={} body={:?}",
                    response_status,
                    response_body,
                );
            } else {
                tracing::debug!(
                    "Got non-success response to recorder start request status={} body={:?}",
                    response_status,
                    response_body,
                );
                return Some(LoopBackEvent::RecorderRequestFailed);
            }

            None
        });

        Ok(())
    }

    fn start_recording(
        &mut self,
        ctx: &mut ModuleContext<'_, RecordingModule>,
        participant_id: ParticipantId,
    ) -> Result<(), SignalingModuleError<RecordingError>> {
        if !ctx.is_moderator(participant_id) {
            return Err(RecordingError::InsufficientPermissions.into());
        }

        if !self.can_record {
            return Err(RecordingError::RecordFeatureDisabled.into());
        }

        match self.recording_states.entry(ctx.room) {
            Entry::Occupied(mut occupied_entry) => {
                if occupied_entry.get().is_running() {
                    return Err(RecordingError::RecordingAlreadyActive.into());
                }

                // If the recording is already requested, we don't need to request it again, just
                // let the requesting participant know that it has been requested
                if occupied_entry.get().is_requested() {
                    ctx.send_ws_message(
                        [participant_id],
                        RecordingEvent::RecordingUpdated(RecordingStatus::Requested),
                    )?;

                    return Ok(());
                }

                occupied_entry.insert(RecordingStatus::Requested);
            }
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(RecordingStatus::Requested);
            }
        }

        let request_result = if let Some((recorder, _)) = find_recorder(ctx) {
            ctx.send_ws_message(
                [*recorder],
                RecordingEvent::Service {
                    event: RecordingServiceCommand::StartRecording,
                },
            )
            .map_err(SignalingModuleError::from)
        } else {
            self.request_recorder(ctx)
        };

        if let Err(e) = request_result {
            // Reset status back to inactive
            self.recording_states
                .insert(ctx.room, RecordingStatus::Inactive);

            return Err(e);
        }

        ctx.send_ws_message(
            ctx.participants.connected().room(ctx.room).ids(),
            RecordingEvent::RecordingUpdated(RecordingStatus::Requested),
        )?;

        Ok(())
    }

    fn pause_recording(
        &mut self,
        ctx: &mut ModuleContext<'_, RecordingModule>,
        participant_id: ParticipantId,
    ) -> Result<(), SignalingModuleError<RecordingError>> {
        if !ctx.is_moderator(participant_id) {
            return Err(RecordingError::InsufficientPermissions.into());
        }

        match self.recording_states.entry(ctx.room) {
            Entry::Occupied(mut occupied_entry) => {
                occupied_entry.insert(RecordingStatus::Paused);
            }
            Entry::Vacant(..) => return Err(RecordingError::RecordingNotActive.into()),
        }

        let (recorder, _) = find_recorder(ctx)
            .context("Invalid state, set recording to paused but no recorder is connected")?;

        ctx.send_ws_message(
            [*recorder],
            RecordingEvent::Service {
                event: RecordingServiceCommand::PauseRecording,
            },
        )?;

        Ok(())
    }

    fn stop_recording(
        &mut self,
        ctx: &mut ModuleContext<'_, RecordingModule>,
        participant_id: ParticipantId,
    ) -> Result<(), SignalingModuleError<RecordingError>> {
        if !ctx.is_moderator(participant_id) {
            return Err(RecordingError::InsufficientPermissions.into());
        }

        if self
            .recording_states
            .get(&ctx.room)
            .is_none_or(|s| !s.is_running())
        {
            return Err(RecordingError::RecordingNotActive.into());
        }

        let (recorder, _) = find_recorder(ctx)
            .context("Failed to find recorder in conference despite an active recording")?;

        ctx.send_ws_message(
            [*recorder],
            RecordingEvent::Service {
                event: RecordingServiceCommand::StopRecording,
            },
        )?;

        Ok(())
    }

    fn start_stream(
        &mut self,
        ctx: &mut ModuleContext<'_, RecordingModule>,
        participant_id: ParticipantId,
        target_ids: BTreeSet<StreamingTargetId>,
    ) -> Result<(), SignalingModuleError<RecordingError>> {
        if !ctx.is_moderator(participant_id) {
            return Err(RecordingError::InsufficientPermissions.into());
        }

        if !self.can_stream {
            return Err(RecordingError::StreamFeatureDisabled.into());
        }

        // Validate target ids, must reference a streaming target & it may not be already active
        for target_id in &target_ids {
            if !self.streaming_targets.iter().any(|c| c.id == *target_id) {
                return Err(RecordingError::InvalidStreamingId.into());
            }

            if self
                .stream_states
                .get(target_id)
                .is_some_and(|(_, status)| !status.can_be_started())
            {
                return Err(RecordingError::StreamingTargetInUse.into());
            }
        }

        // Set the stream config to be in use
        for target_id in &target_ids {
            self.stream_states
                .insert(*target_id, (ctx.room, StreamStatus::Requested));
        }

        let request_result = if let Some((recorder, _)) = find_recorder(ctx) {
            ctx.send_ws_message(
                [*recorder],
                RecordingEvent::Service {
                    event: RecordingServiceCommand::StartStreams {
                        target_ids: target_ids.clone(),
                    },
                },
            )
            .map_err(SignalingModuleError::from)
        } else {
            self.request_recorder(ctx)
        };

        if let Err(e) = request_result {
            // Remove stream states since request was unsuccessful
            for target_id in &target_ids {
                self.stream_states.remove(target_id);
            }

            return Err(e);
        }

        for target_id in target_ids {
            ctx.send_ws_message(
                ctx.participants.in_room(ctx.room).connected().ids(),
                RecordingEvent::StreamUpdated {
                    target_id,
                    status: StreamStatus::Requested,
                },
            )?;

            ctx.send_ws_message(
                ctx.participants
                    .connected()
                    .iter()
                    .filter(|(_, state)| state.room != ctx.room)
                    .map(|(id, _)| *id),
                RecordingEvent::StreamUpdated {
                    target_id,
                    status: StreamStatus::InUse,
                },
            )?;
        }

        Ok(())
    }

    fn pause_stream(
        &mut self,
        ctx: &mut ModuleContext<'_, RecordingModule>,
        participant_id: ParticipantId,
        target_ids: BTreeSet<StreamingTargetId>,
    ) -> Result<(), SignalingModuleError<RecordingError>> {
        if !ctx.is_moderator(participant_id) {
            return Err(RecordingError::InsufficientPermissions.into());
        }

        // Validate target ids it must be running in the current room
        for target_id in &target_ids {
            if !self
                .stream_states
                .get(target_id)
                .is_some_and(|(room, status)| *room == ctx.room && status.is_running())
            {
                return Err(RecordingError::InvalidStreamingId.into());
            }
        }

        let (recorder, _) = find_recorder(ctx)
            .context("Failed to find recorder in conference despite having active streams")?;

        ctx.send_ws_message(
            [*recorder],
            RecordingEvent::Service {
                event: RecordingServiceCommand::PauseStreams { target_ids },
            },
        )?;

        Ok(())
    }

    fn stop_stream(
        &mut self,
        ctx: &mut ModuleContext<'_, RecordingModule>,
        participant_id: ParticipantId,
        target_ids: BTreeSet<StreamingTargetId>,
    ) -> Result<(), SignalingModuleError<RecordingError>> {
        if !ctx.is_moderator(participant_id) {
            return Err(RecordingError::InsufficientPermissions.into());
        }

        // Validate target ids, must reference a streaming target & it must be running
        for target_id in &target_ids {
            if !self
                .stream_states
                .get(target_id)
                .is_some_and(|(room, status)| *room == ctx.room && status.is_running())
            {
                return Err(RecordingError::InvalidStreamingId.into());
            }
        }

        let (recorder, _) = find_recorder(ctx)
            .context("Failed to find recorder in conference despite having active streams")?;

        ctx.send_ws_message(
            [*recorder],
            RecordingEvent::Service {
                event: RecordingServiceCommand::StopStreams { target_ids },
            },
        )?;

        Ok(())
    }

    fn handle_service_message(
        &mut self,
        ctx: &mut ModuleContext<'_, RecordingModule>,
        participant_id: ParticipantId,
        command: RecordingServiceEvent,
    ) -> Result<(), SignalingModuleError<RecordingError>> {
        if !ctx
            .participant_state(participant_id)
            .is_some_and(|state| state.kind == ClientKind::Recorder { room: ctx.room })
        {
            return Err(RecordingError::InsufficientPermissions.into());
        }

        match command {
            RecordingServiceEvent::RecordingUpdated(status) => {
                let Some(state) = self.recording_states.get_mut(&ctx.room) else {
                    return Err(RecordingError::InvalidStreamingId.into());
                };

                *state = status.clone();

                ctx.send_ws_message(
                    ctx.participants.in_room(ctx.room).connected().ids(),
                    RecordingEvent::RecordingUpdated(status),
                )?;

                Ok(())
            }
            RecordingServiceEvent::StreamUpdated { target_id, status } => {
                let Some((room, current_status)) = self.stream_states.get_mut(&target_id) else {
                    return Err(RecordingError::InvalidStreamingId.into());
                };

                if *room != ctx.room {
                    return Err(RecordingError::InvalidStreamingId.into());
                }

                ctx.send_ws_message(
                    ctx.participants.in_room(ctx.room).connected().ids(),
                    RecordingEvent::StreamUpdated {
                        target_id,
                        status: status.clone(),
                    },
                )?;

                // Participants in other rooms must be updated if the StreamingTarget is now in use
                if current_status.can_be_started() != status.can_be_started() {
                    let status = if status.can_be_started() {
                        StreamStatus::Inactive
                    } else {
                        StreamStatus::InUse
                    };

                    ctx.send_ws_message(
                        ctx.participants
                            .connected()
                            .iter()
                            .filter(|(_, state)| state.room != ctx.room)
                            .map(|(id, _)| *id),
                        RecordingEvent::StreamUpdated { target_id, status },
                    )?;
                }

                *current_status = status;

                Ok(())
            }
        }
    }

    /// Resets the recording state & stream states back to inactive if no recorder is present in the
    /// room
    fn reset_states_if_no_recorder_is_connected(
        &mut self,
        ctx: &mut ModuleContext<'_, RecordingModule>,
    ) -> Result<(), SignalingModuleError<RecordingError>> {
        if find_recorder(ctx).is_some() {
            return Ok(());
        }

        if self
            .recording_states
            .remove(&ctx.room)
            .is_some_and(|state| state != RecordingStatus::Inactive)
        {
            ctx.send_ws_message(
                ctx.participants.in_room(ctx.room).connected().ids(),
                RecordingEvent::RecordingUpdated(RecordingStatus::Inactive),
            )?;
        }

        // Find all stream targets to reset to inactive
        let target_ids: Vec<_> = self
            .stream_states
            .iter()
            .filter(|(_, (room, status))| *room == ctx.room && *status != StreamStatus::Inactive)
            .map(|(target_id, _)| *target_id)
            .collect();

        for target_id in target_ids {
            self.stream_states.remove(&target_id);

            ctx.send_ws_message(
                ctx.participants.in_room(ctx.room).connected().ids(),
                RecordingEvent::StreamUpdated {
                    target_id,
                    status: StreamStatus::Inactive,
                },
            )?;
        }

        Ok(())
    }
}

fn find_recorder<'a>(
    ctx: &'a ModuleContext<'a, RecordingModule>,
) -> Option<(&'a ParticipantId, &'a ParticipantState)> {
    ctx.participants.in_room(ctx.room).iter().find(|(_, p)| {
        p.is_connected() && matches!(p.kind, ClientKind::Recorder { room } if room == ctx.room)
    })
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_roomserver_signaling::signaling_module::SignalingModuleDescription as _;

    use crate::RecordingModule;

    #[test]
    fn serialize_example_config() {
        let produced = toml::to_string_pretty(&RecordingModule::example_config()).unwrap();

        assert_snapshot!(produced, @r#"
        [recording]
        url = "http://localhost:5555/"

        [recording.api_key]
        id = "recorder"
        secret = "secret"
        "#);
    }
}

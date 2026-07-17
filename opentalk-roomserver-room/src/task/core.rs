// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, hash_map::Entry},
    sync::Arc,
};

use anyhow::{Context, anyhow};
use futures::SinkExt as _;
use opentalk_roomserver_signaling::{
    event_origin::{EventOrigin, ParticipantOrigin},
    module_context::ModuleMessage,
    participant_state::ParticipantState,
};
use opentalk_roomserver_types::{
    client_parameters::{ClientKind, Role as RoomserverClientRole},
    connection_id::ConnectionId,
    core::{
        CORE_MODULE_ID, CoreCommand, CoreError, CoreEvent, JoinBlockedReason, LeftWaitingRoom,
        RoomCloseReason, state::CoreState,
    },
    device_id::DeviceId,
    disconnect_reason::DisconnectReason,
    error::SignalingError,
    join::{
        connection_info::ConnectionInfo, event_info::EventInfo, join_success::JoinSuccess,
        participant::Participant,
    },
    room_info::RoomInfo,
    room_kind::RoomKind,
    room_parameters_patch::RoomParametersPatch,
    shared_json::SharedJson,
    signaling::{
        SignalingCommand,
        module_error::{FatalError, SignalingModuleError},
        websocket::{CloseFrame, SignalingSocket, SignalingSocketMessage},
    },
};
use opentalk_types_api_internal::module_assets::Quota;
use opentalk_types_common::{
    events::MeetingDetails, features::FeatureId, modules::ModuleId, streaming::StreamingLink,
    tariffs::QuotaType, time::Timestamp,
};
use opentalk_types_signaling::{ModuleData, ParticipantId, Role};

use super::RoomTask;
use crate::{
    signaling::{CORE_MODULES, DynBroadcastEvent, dyn_module_context::DynModuleContext},
    task::participant_id_from_uuid,
};

impl<Socket: SignalingSocket> RoomTask<Socket> {
    pub(crate) fn handle_conference_core_command(
        &mut self,
        participant_origin: ParticipantOrigin,
        command: SignalingCommand,
    ) {
        let core_command: CoreCommand = match serde_json::from_str(command.payload.get()) {
            Ok(command) => command,
            Err(err) => {
                tracing::debug!("received unsupported core command from conference");
                self.message_router.conference.send_error(
                    participant_origin.connection_id,
                    participant_origin.transaction_id,
                    SignalingError::InvalidJson {
                        message: format!("{err:?}"),
                    },
                );
                return;
            }
        };

        let result = match core_command {
            CoreCommand::EnterRoom => self.message_router.conference.serialize_and_send(
                [participant_origin.connection_id],
                CORE_MODULE_ID,
                command.transaction_id,
                CoreEvent::Error(CoreError::AlreadyInRoom),
            ),
        };

        if let Err(err) = result {
            tracing::error!("fatal error in core module: {err:?}");
            self.message_router.conference.send_error(
                participant_origin.connection_id,
                command.transaction_id,
                SignalingError::Internal,
            );
        }
    }

    pub(crate) fn handle_waiting_room_core_command(
        &mut self,
        participant_origin: ParticipantOrigin,
        command: SignalingCommand,
    ) {
        let core_command: CoreCommand = match serde_json::from_str(command.payload.get()) {
            Ok(command) => command,
            Err(err) => {
                tracing::debug!("received unsupported core command from waiting room");
                self.message_router.waiting_room.send_error(
                    participant_origin.connection_id,
                    participant_origin.transaction_id,
                    SignalingError::InvalidJson {
                        message: format!("{err:?}"),
                    },
                );
                return;
            }
        };

        let result = match core_command {
            CoreCommand::EnterRoom => self.enter_room(participant_origin),
        };

        let router = self.message_router_for_participant(participant_origin.id);
        if let Err(e) = result {
            match e {
                SignalingModuleError::Internal(err) => {
                    tracing::error!("internal error in core module: {err:?}");

                    router.send_error(
                        participant_origin.connection_id,
                        command.transaction_id,
                        SignalingError::Internal,
                    );
                }
                SignalingModuleError::Fatal(err) => {
                    tracing::error!("fatal error in core module: {err:?}");

                    router.send_error(
                        participant_origin.connection_id,
                        command.transaction_id,
                        SignalingError::Internal,
                    );
                }
                SignalingModuleError::Module(module_error) => {
                    let result = router.serialize_and_send(
                        [participant_origin.connection_id],
                        CORE_MODULE_ID,
                        command.transaction_id,
                        CoreEvent::Error(module_error),
                    );

                    if let Err(fatal_error) = result {
                        tracing::error!("failed to send error in core module: {fatal_error:?}");

                        router.send_error(
                            participant_origin.connection_id,
                            command.transaction_id,
                            SignalingError::Internal,
                        );
                    }
                }
                SignalingModuleError::NotSupported => {
                    router.send_error(
                        participant_origin.connection_id,
                        command.transaction_id,
                        SignalingError::NotSupported,
                    );
                }
            }
        }
    }

    fn enter_room(
        &mut self,
        participant_origin: ParticipantOrigin,
    ) -> Result<(), SignalingModuleError<CoreError>> {
        let Entry::Occupied(participant) = self.waiting_participants.entry(participant_origin.id)
        else {
            tracing::debug!("Failed to enter room: participant not known");
            return Err(CoreError::UnknownParticipant.into());
        };

        if !participant.get().accepted {
            tracing::debug!("Failed to enter room: participant not yet accepted");
            return Err(CoreError::NotAccepted.into());
        }
        let participant = participant.remove();

        let moderator_ids = self.participants.connected().moderators().connection_ids();

        self.message_router.conference.serialize_and_send(
            moderator_ids,
            CORE_MODULE_ID,
            None,
            CoreEvent::LeftWaitingRoom(LeftWaitingRoom {
                id: participant_origin.id,
                connection_ids: participant.connections.keys().copied().collect(),
            }),
        )?;

        self.message_router
            .upgrade_connections(participant.connections.keys());

        for (&connection_id, &device_id) in &participant.connections {
            self.join_room(
                participant_origin.id,
                connection_id,
                device_id,
                participant.kind.clone(),
                participant.role,
            )?;
        }

        Ok(())
    }

    /// A participant connected to the conference
    ///
    /// Sends the [`CoreEvent::JoinSuccess`] to the connection of the participant that joins.
    /// Notifies other connections with the [`CoreEvent::ParticipantConnected`] message.
    ///
    /// NOTE: In case the joining participant is already connected with another device, they will
    /// also receive the [`CoreEvent::ParticipantConnected`] messages on the device that is already
    /// connected.
    pub(super) fn participant_joined(
        &mut self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        device_id: DeviceId,
        client_kind: ClientKind,
        role: RoomserverClientRole,
    ) -> Result<(), FatalError> {
        let mut own_data = ModuleData::new();
        let mut peer_events = BTreeMap::new();
        let mut peer_data = BTreeMap::new();

        let Some(current_breakout_room) = self
            .participants
            .all_unfiltered
            .get(&participant_id)
            .map(|s| s.room)
        else {
            return Err(FatalError(anyhow!(
                "Unable to get participant state for fresh connections"
            )));
        };

        let participant_origin = ParticipantOrigin {
            id: participant_id,
            connection_id,
            transaction_id: None,
        };

        let actions = self.broadcast_event_to_modules(
            participant_origin.into(),
            current_breakout_room,
            DynBroadcastEvent::Connected {
                participant_id,
                connection_id,
                own_data: &mut own_data,
                peer_events: &mut peer_events,
                peer_data: &mut peer_data,
            },
        );

        self.join_success_breakout_own_data(&mut own_data, current_breakout_room);
        self.join_success_breakout_peer_events(participant_id, &mut peer_events)?;
        self.join_success_core_peer_events(participant_id, &mut peer_events)?;

        let join_success_msg = self.build_join_success(
            participant_id,
            connection_id,
            device_id,
            client_kind,
            role,
            own_data,
            peer_data.clone(),
        )?;

        self.message_router.conference.serialize_and_send(
            [connection_id],
            CORE_MODULE_ID,
            None,
            CoreEvent::JoinSuccess(Box::new(join_success_msg)),
        )?;

        for (&peer_id, state) in self.participants.connected().iter() {
            let peer_join_info = peer_events.remove(&peer_id);

            let connections = state
                .connections
                .keys()
                .copied()
                .filter(|&c| c != connection_id);

            self.message_router.conference.serialize_and_send(
                connections,
                CORE_MODULE_ID,
                None,
                CoreEvent::ParticipantConnected {
                    participant_id,
                    connection_id,
                    peer_data: peer_join_info.unwrap_or_default(),
                },
            )?;
        }

        actions.handle_requested_messages(self)?;

        // Start the quota timer if the room has a time limit
        if self
            .info
            .room
            .tariff
            .quotas
            .contains_key(&QuotaType::RoomTimeLimitSecs)
        {
            // This will have no effect if the timer is already running
            self.quota_timeout.start();
        }

        Ok(())
    }

    pub(super) fn participant_limit_reached(mut socket: Socket) -> Result<(), FatalError> {
        const WS_CLOSE_TRY_AGAIN_LATER: u16 = 1013;
        let event = CoreEvent::JoinBlocked {
            reason: JoinBlockedReason::ParticipantLimitReached,
        };
        let msg = serde_json::to_string(&event)
            .context("Failed to serialize `JoinBlocked` event")
            .map_err(FatalError)?;
        tokio::spawn(async move {
            socket.send(SignalingSocketMessage::Text(msg)).await?;
            socket
                .send(SignalingSocketMessage::Close(Some(CloseFrame {
                    code: WS_CLOSE_TRY_AGAIN_LATER,
                    reason: "Participant limit reached".into(),
                })))
                .await
        });

        Ok(())
    }

    /// Inform modules that the participant has left the conference and broadcast
    /// [`CoreEvent::ParticipantDisconnected`] to all participants
    pub(super) fn participant_disconnected(
        &mut self,
        origin: EventOrigin,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        room: RoomKind,
        reason: DisconnectReason,
    ) -> Result<(), FatalError> {
        self.broadcast_event_to_modules(
            origin,
            room,
            DynBroadcastEvent::Disconnected {
                participant_id,
                connection_id,
            },
        )
        .handle_requested_messages(self)?;

        let content = CoreEvent::ParticipantDisconnected {
            participant_id,
            connection_id,
            reason,
        };

        self.message_router
            .conference
            .serialize_and_broadcast(CORE_MODULE_ID, None, content)?;

        Ok(())
    }

    #[tracing::instrument(skip(self, event), fields(%event))]
    pub(crate) fn broadcast_event_to_modules(
        &mut self,
        origin: EventOrigin,
        room_kind: RoomKind,
        mut event: DynBroadcastEvent<'_>,
    ) -> RequestedModuleActions {
        let mut errors = Vec::new();
        let mut messages = RefCell::new(Vec::new());
        let timestamp = Timestamp::now();
        for (namespace, module) in &mut self.modules {
            if let Err(err) = module.on_broadcast_event(
                &mut DynModuleContext::new(
                    self.info.room_id,
                    room_kind,
                    origin,
                    &mut self.info,
                    &mut self.participants,
                    &mut self.waiting_participants,
                    &mut self.banned_participants,
                    timestamp,
                    Arc::clone(&self.ctx.asset_storage),
                    Arc::clone(&self.ctx.module_resources),
                    &mut messages,
                    &mut self.loopback_futures,
                ),
                &mut event,
            ) {
                errors.push((namespace.clone(), err));
            }
        }

        RequestedModuleActions {
            messages,
            errors,
            timestamp,
            room_kind,
            origin,
        }
    }

    /// An unrecoverable module error occurred and the module needs to be removed for the remainder
    /// of the conference
    ///
    /// Further requests to the module will result in a [`SignalingError::UnknownNamespace`] error.
    pub(crate) fn handle_fatal_module_error(
        &mut self,
        namespace: ModuleId,
        transaction_id: Option<u64>,
        err: FatalError,
    ) {
        tracing::error!(
            "The {namespace} module caused a fatal error and will be shut down: {:#?}",
            err.0
        );

        let Some(mut module) = self.modules.remove(&namespace) else {
            tracing::error!("Attempted to remove non-existent module {namespace}");
            return;
        };

        let timestamp = Timestamp::now();
        let mut messages = RefCell::new(Vec::new());
        let mut ctx = DynModuleContext::new(
            self.info.room_id,
            RoomKind::Main,
            EventOrigin::Internal,
            &mut self.info,
            &mut self.participants,
            &mut self.waiting_participants,
            &mut self.banned_participants,
            timestamp,
            Arc::clone(&self.ctx.asset_storage),
            Arc::clone(&self.ctx.module_resources),
            &mut messages,
            &mut self.loopback_futures,
        );
        module.on_closing(&mut ctx);

        if let Err(err) =
            self.handle_module_messages(messages, RoomKind::Main, EventOrigin::Internal, timestamp)
        {
            tracing::error!("Handling module messages during fatal error handling failed: {err:?}");
        }

        // Remove the module from the room state
        self.info.room.module_settings.remove(&namespace);

        self.message_router.conference.broadcast_error(
            transaction_id,
            SignalingError::FatalModuleError { namespace },
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn build_join_success(
        &self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        device_id: DeviceId,
        client_kind: ClientKind,
        role: RoomserverClientRole,
        module_data: ModuleData,
        mut participants_module_data: BTreeMap<ParticipantId, BTreeMap<ModuleId, SharedJson>>,
    ) -> Result<JoinSuccess, FatalError> {
        let participants = self
            .participants
            .all_unfiltered
            .iter()
            .filter(|(id, ..)| id != &&participant_id)
            .map(|(id, state)| {
                let connections = state
                    .connections
                    .iter()
                    .map(|(conn, device)| ConnectionInfo {
                        connection_id: *conn,
                        device_id: *device,
                    })
                    .collect();

                let mut module_peer_data = participants_module_data.remove(id).unwrap_or_default();
                Self::join_success_breakout_peer_data(&mut module_peer_data, state)?;
                self.join_success_core_peer_data(*id, state, &mut module_peer_data)?;

                Ok(Participant {
                    id: *id,
                    connections,
                    module_data: module_peer_data,
                })
            })
            .collect::<Result<Vec<_>, FatalError>>()?;

        let display_name = self
            .participants
            .all_unfiltered
            .get(&participant_id)
            .map(|state| state.kind.display_name())
            .unwrap_or_else(|| client_kind.display_name());
        let (role, avatar_url, is_room_owner) = match client_kind {
            ClientKind::Registered { profile } | ClientKind::RegisteredCallIn { profile } => (
                role.to_opentalk_types_signaling_role(),
                Some(profile.user_info.avatar_url),
                self.info.room.created_by.id == profile.id,
            ),
            ClientKind::Guest { .. }
            | ClientKind::Transcription { .. }
            | ClientKind::Recorder { .. }
            | ClientKind::CallIn { .. } => (Role::Guest, None, false),
        };

        let event_info = self
            .info
            .room
            .event
            .as_ref()
            .map(|event_context| EventInfo {
                id: event_context.id,
                room_id: self.info.room_id,
                title: event_context.title.clone(),
                is_adhoc: event_context.is_adhoc,
                e2e_encryption: self.info.room.e2e_encryption,
            });

        let meeting_details = MeetingDetails {
            invite_code_id: self.info.room.invite_code,
            call_in: self.info.room.call_in.clone(),
            streaming_links: self
                .info
                .room
                .streaming_targets
                .iter()
                .filter_map(|target| {
                    let url = target.streaming_target.kind.get_stream_target_location()?;

                    Some(StreamingLink {
                        name: target.streaming_target.name.clone(),
                        url,
                    })
                })
                .collect(),
        };

        let other_connections = self
            .participants
            .all_unfiltered
            .get(&participant_id)
            .map(|state| {
                state
                    .connections
                    .iter()
                    .filter(|(conn, ..)| conn != &&connection_id)
                    .map(|(conn, device)| ConnectionInfo {
                        connection_id: *conn,
                        device_id: *device,
                    })
                    .collect()
            })
            .unwrap_or_default();

        let enabled_modules = self.enabled_modules();

        Ok(JoinSuccess {
            id: participant_id,
            connection_id,
            device_id,
            connections: other_connections,
            display_name,
            avatar_url,
            role,
            closes_at: self.info.closes_at,
            tariff: Box::new(self.info.room.tariff.clone()),
            participants,
            event_info,
            room_info: RoomInfo {
                id: self.info.room_id,
                password: self.info.room.password.clone(),
                created_by: self.info.room.created_by.user_info.clone(),
            },
            meeting_details,
            is_room_owner,
            module_data,
            enabled_modules,
        })
    }

    /// Return the list of enabled modules and their enabled features.
    ///
    /// Enabled modules are calculated by
    ///
    /// * the list of configured modules in the [`RoomParameters`]
    /// * adding the mandatory modules: [`CORE_MODULES`]
    /// * disabling features in [`TariffDetails::disabled_features`]
    ///
    /// [`TariffDetails::disabled_features`]: opentalk_roomserver_types::tariff_details::TariffDetails::disabled_features
    /// [`RoomParameters`]: opentalk_roomserver_types::room_parameters::RoomParameters
    pub(crate) fn enabled_modules(&self) -> BTreeMap<ModuleId, BTreeSet<FeatureId>> {
        let mut enabled_modules = self.ctx.module_registry.module_features();

        enabled_modules.retain(|module_id, _| {
            self.modules.contains_key(module_id) || CORE_MODULES.contains(module_id)
        });

        for module_feature in &self.info.room.tariff.disabled_features {
            enabled_modules
                .get_mut(&module_feature.module)
                .map(|features| features.remove(&module_feature.feature));
        }
        enabled_modules
    }

    /// Attach core related info about the joining participant for other participants (peers).
    ///
    /// This will be sent to all other participants, but not the joining participant.
    pub(crate) fn join_success_core_peer_events(
        &self,
        participant_id: ParticipantId,
        peer_events: &mut BTreeMap<ParticipantId, BTreeMap<ModuleId, SharedJson>>,
    ) -> Result<(), FatalError> {
        let Some(joinee) = self.participants.all_unfiltered.get(&participant_id) else {
            return Err(FatalError(anyhow::anyhow!("joining participant not found")));
        };
        let data = CoreState {
            display_name: joinee.kind.display_name(),
            role: joinee.role,
            avatar_url: joinee.kind.avatar_url().map(|s| s.to_owned()),
            participation_kind: joinee.kind.participant_kind(),
            joined_at: joinee.joined_at.into(),
            left_at: joinee.left_at.map(Into::into),
            is_room_owner: participant_id_from_uuid(self.info.owner()) == participant_id,
        };
        let data = SharedJson::from(
            serde_json::to_value(data)
                .context("Failed to serialize CoreState")
                .map_err(FatalError)?,
        );
        for peer_id in self.participants.connected().ids() {
            peer_events
                .entry(peer_id)
                .or_default()
                .insert(CORE_MODULE_ID, data.clone());
        }
        Ok(())
    }

    /// Attach core related info about the other participant (`peer`) for the joining participant.
    ///
    /// This will be sent to the joining participant.
    pub(crate) fn join_success_core_peer_data(
        &self,
        peer: ParticipantId,
        state: &ParticipantState,
        peer_data: &mut BTreeMap<ModuleId, SharedJson>,
    ) -> Result<(), FatalError> {
        let data = CoreState {
            display_name: state.kind.display_name(),
            role: state.role,
            avatar_url: state.kind.avatar_url().map(str::to_owned),
            participation_kind: state.kind.participant_kind(),
            joined_at: state.joined_at.into(),
            left_at: state.left_at.map(Into::into),
            is_room_owner: participant_id_from_uuid(self.info.owner()) == peer,
        };
        let data = SharedJson::from(
            serde_json::to_value(data)
                .context("Failed to serialize CoreState")
                .map_err(FatalError)?,
        );
        peer_data.insert(CORE_MODULE_ID, data);

        Ok(())
    }

    pub(crate) fn broadcast_room_parameters_changed_event(
        &mut self,
        change: RoomParametersPatch,
    ) -> Result<(), FatalError> {
        tracing::trace!("Broadcasting room parameters changed event");
        let connections = if self.info.room.show_meeting_details {
            self.participants.connected().connection_ids()
        } else {
            self.participants.moderators().connected().connection_ids()
        };

        self.message_router.conference.serialize_and_send(
            connections,
            CORE_MODULE_ID,
            None,
            CoreEvent::RoomParametersChanged { change },
        )
    }

    /// Notify moderators that the storage quota changed
    pub(crate) fn broadcast_storage_quota_changed_event(
        &mut self,
        quota: Quota,
    ) -> Result<(), FatalError> {
        tracing::trace!("Broadcasting storage quota changed event");

        self.message_router.conference.serialize_and_send(
            self.participants.moderators().connected().connection_ids(),
            CORE_MODULE_ID,
            None,
            CoreEvent::StorageQuotaChanged { quota },
        )
    }

    pub(crate) fn broadcast_closing_event(
        &mut self,
        reason: RoomCloseReason,
    ) -> Result<(), FatalError> {
        tracing::trace!("Broadcasting close notification to participants");
        let event = CoreEvent::Closing { reason };
        self.message_router.conference.serialize_and_broadcast(
            CORE_MODULE_ID,
            None,
            event.clone(),
        )?;

        self.message_router
            .waiting_room
            .serialize_and_broadcast(CORE_MODULE_ID, None, event)?;

        Ok(())
    }
}

#[must_use]
pub(crate) struct RequestedModuleActions {
    pub messages: RefCell<Vec<ModuleMessage>>,
    pub errors: Vec<(ModuleId, FatalError)>,
    pub timestamp: Timestamp,
    pub room_kind: RoomKind,
    pub origin: EventOrigin,
}

impl RequestedModuleActions {
    pub(crate) fn handle_requested_messages(
        self,
        task: &mut RoomTask<impl SignalingSocket>,
    ) -> Result<(), FatalError> {
        task.handle_module_messages(self.messages, self.room_kind, self.origin, self.timestamp)?;

        for (namespace, err) in self.errors {
            task.handle_fatal_module_error(namespace, self.origin.transaction_id(), err);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use insta::assert_snapshot;
    use opentalk_roomserver_signaling::signaling_event::SignalingEvent;
    use opentalk_roomserver_types::{
        connection_id::ConnectionId,
        core::CORE_MODULE_ID,
        device_id::DeviceId,
        join::{connection_info::ConnectionInfo, join_success::JoinSuccess},
        room_info::RoomInfo,
        tariff_details::TariffDetails,
    };
    use opentalk_types_common::{
        events::MeetingDetails,
        modules::{ModuleId, module_id},
        rooms::RoomId,
        time::Timestamp,
        users::{DisplayName, UserInfo},
        utils::ExampleData,
    };
    use opentalk_types_signaling::{ModuleData, ParticipantId, Role, SignalingModuleFrontendData};
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use super::CoreEvent;

    fn test_module_data() -> ModuleData {
        #[derive(Debug, Serialize, Deserialize)]
        struct TestData {
            a: String,
            b: u64,
        }

        impl SignalingModuleFrontendData for TestData {
            const NAMESPACE: Option<ModuleId> = Some(module_id!("test"));
        }

        let mut module_data = ModuleData::new();

        module_data
            .insert(&TestData {
                a: "test".into(),
                b: 42,
            })
            .unwrap();

        module_data
    }

    #[test]
    fn serialize_signaling_event_success() {
        let join_success = JoinSuccess {
            id: ParticipantId::nil(),
            connection_id: ConnectionId::from_u128(0x1),
            device_id: DeviceId::from_u128(0x2),
            connections: vec![ConnectionInfo {
                connection_id: ConnectionId::from_u128(0x3),
                device_id: DeviceId::from_u128(0x4),
            }],
            display_name: DisplayName::example_data(),
            avatar_url: None,
            role: Role::Guest,
            closes_at: None,
            tariff: Box::new(TariffDetails::example_data()),
            module_data: test_module_data(),
            participants: vec![],
            event_info: None,
            room_info: RoomInfo {
                id: RoomId::nil(),
                password: None,
                created_by: UserInfo::example_data(),
            },
            meeting_details: MeetingDetails {
                invite_code_id: None,
                call_in: None,
                streaming_links: vec![],
            },
            is_room_owner: false,
            enabled_modules: BTreeMap::from_iter([(
                module_id!("test_module"),
                BTreeSet::from_iter([]),
            )]),
        };
        let event = SignalingEvent {
            namespace: CORE_MODULE_ID,
            timestamp: Timestamp::unix_epoch(),
            payload: CoreEvent::JoinSuccess(join_success.into()),
            transaction_id: Some(0),
        };

        let produced = serde_json::to_string_pretty(&event).unwrap();
        assert_snapshot!(produced, @r#"
        {
          "namespace": "core",
          "transaction_id": 0,
          "timestamp": "1970-01-01T00:00:00Z",
          "payload": {
            "message": "join_success",
            "id": "00000000-0000-0000-0000-000000000000",
            "connection_id": "00000000-0000-0000-0000-000000000001",
            "device_id": "00000000-0000-0000-0000-000000000002",
            "connections": [
              {
                "connection_id": "00000000-0000-0000-0000-000000000003",
                "device_id": "00000000-0000-0000-0000-000000000004"
              }
            ],
            "display_name": "Alice Adams",
            "role": "guest",
            "tariff": {
              "id": "00000000-0000-0000-0000-000000000000",
              "name": "Starter tariff",
              "quotas": {
                "max_storage": 50000
              },
              "used_quota": {
                "max_storage": 20000
              },
              "disabled_features": [
                "recording::record"
              ]
            },
            "enabled_modules": {
              "test_module": []
            },
            "module_data": {
              "test": {
                "a": "test",
                "b": 42
              }
            },
            "participants": [],
            "event_info": null,
            "meeting_details": {
              "streaming_links": []
            },
            "room_info": {
              "id": "00000000-0000-0000-0000-000000000000",
              "created_by": {
                "title": "",
                "firstname": "Alice",
                "lastname": "Adams",
                "display_name": "Alice Adams",
                "avatar_url": "https://gravatar.com/avatar/c160f8cc69a4f0bf2b0362752353d060"
              }
            },
            "is_room_owner": false
          }
        }
        "#);

        let json = json!({
            "namespace": "core",
            "transaction_id": 0,
            "timestamp": "1970-01-01T00:00:00Z",
            "payload": {
            "message": "join_success",
                "id": "00000000-0000-0000-0000-000000000000",
                "connection_id": "00000000-0000-0000-0000-000000000000",
                "device_id": "00000000-0000-0000-0000-000000000000",
                "connections": [
                    {
                        "connection_id": "00000000-0000-0000-0000-000000000000",
                        "device_id": "00000000-0000-0000-0000-000000000000"
                    }
                ],
                "display_name": "Alice Adams",
                "role": "guest",
                "tariff": {
                    "id": "00000000-0000-0000-0000-000000000000",
                    "name": "Starter tariff",
                    "quotas": {
                        "max_storage": 50000
                    },
                    "used_quota": {
                        "max_storage": 20000
                    },
                    "disabled_features": [
                        "recording::record"
                    ],
                },
                "enabled_modules": {
                    "test_module": ["test_feature"]
                },
                "module_data": {
                    "test": {
                        "a": "test",
                        "b": 42
                    }
                },
                "participants": [],
                "event_info": null,
                "room_info": {
                    "id": "00000000-0000-0000-0000-000000000000",
                    "created_by": {
                        "title": "",
                        "firstname": "Alice",
                        "lastname": "Adams",
                        "display_name": "Alice Adams",
                        "avatar_url": "https://gravatar.com/avatar/c160f8cc69a4f0bf2b0362752353d060"
                    }
                },
                "meeting_details": {
                    "streaming_links": []
                },

                "is_room_owner": false
            }
        });

        let _: SignalingEvent<CoreEvent> = serde_json::from_value(json).unwrap();
    }
}

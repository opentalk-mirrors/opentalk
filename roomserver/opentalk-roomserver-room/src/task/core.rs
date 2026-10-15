// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, hash_map::Entry},
    str::FromStr,
    sync::Arc,
};

use anyhow::{Context, anyhow};
use chrono::Utc;
use futures::SinkExt as _;
use opentalk_roomserver_signaling::{
    event_origin::{EventOrigin, ParticipantOrigin},
    lobby_participant::{LobbyConnection, LobbyParticipant, WaitingRoomState},
    module_context::ModuleMessage,
    participant_state::ParticipantState,
    signaling_event::SignalingEvent,
};
use opentalk_roomserver_types::{
    client_parameters::{ClientKind, Role as RoomserverClientRole},
    connection_id::ConnectionId,
    core::{
        CORE_MODULE_ID, CoreCommand, CoreError, CoreEvent, JoinBlockedReason, LeftWaitingRoom,
        RoomCloseReason, state::CoreState,
    },
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
    tariffs::QuotaType, time::Timestamp, users::DisplayName,
};
use opentalk_types_signaling::{ModuleData, ParticipantId, Role};

use super::RoomTask;
use crate::{
    signaling::{CORE_MODULES, DynBroadcastEvent, dyn_module_context::DynModuleContext},
    task::participant_id_from_uuid,
};

const DEFAULT_RECORDER_NAME: &str = "recorder";
const DEFAULT_TRANSCRIPTION_NAME: &str = "transcription";

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
            CoreCommand::EnterRoom { .. } | CoreCommand::EnterWaitingRoom { .. } => {
                self.message_router.conference.serialize_and_send(
                    [participant_origin.connection_id],
                    CORE_MODULE_ID,
                    command.transaction_id,
                    CoreEvent::Error(CoreError::AlreadyInRoom),
                )
            }
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

    pub(crate) fn handle_lobby_core_command(
        &mut self,
        participant_origin: ParticipantOrigin,
        command: SignalingCommand,
    ) {
        let core_command: CoreCommand = match serde_json::from_str(command.payload.get()) {
            Ok(command) => command,
            Err(err) => {
                tracing::debug!("received unsupported core command from waiting room");
                self.message_router.lobby.send_error(
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
            CoreCommand::EnterRoom { display_name } => {
                self.enter_room(participant_origin, display_name)
            }
            CoreCommand::EnterWaitingRoom { display_name } => {
                self.enter_waiting_room(participant_origin, display_name)
            }
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

    fn enter_waiting_room(
        &mut self,
        participant_origin: ParticipantOrigin,
        display_name: Option<DisplayName>,
    ) -> Result<(), SignalingModuleError<CoreError>> {
        let Some(participant) = self.lobby_participants.get_mut(&participant_origin.id) else {
            tracing::debug!("Failed to enter waiting room: participant not known");
            return Err(CoreError::UnknownParticipant.into());
        };

        if participant.accepted {
            tracing::debug!("Failed to enter waiting room: participant already accepted");
            return Err(CoreError::AlreadyAccepted.into());
        }

        let conference = self.participants.connected().get(&participant_origin.id);
        let mask_phone_numbers = self.info.mask_unmapped_phone_numbers();
        let default_country_code = self.info.default_country_code();

        let display_name = Self::require_display_name(
            conference,
            participant,
            display_name,
            mask_phone_numbers,
            default_country_code,
        )?;

        let connection_id = participant_origin.connection_id;
        let Some(connection) = participant.connection_mut(connection_id) else {
            tracing::debug!("Failed to enter waiting room: connection not known");
            return Err(CoreError::UnknownConnection.into());
        };

        if connection.in_waiting_room {
            tracing::debug!("Connection already in waiting room");
            self.message_router.lobby.serialize_and_send(
                [connection_id],
                CORE_MODULE_ID,
                participant_origin.transaction_id,
                CoreEvent::InWaitingRoom,
            )?;

            return Ok(());
        }

        connection.in_waiting_room = true;
        let joined_at = Utc::now();
        participant.waiting_room_state = Some(WaitingRoomState {
            display_name: display_name.clone(),
            joined_at,
        });

        // Notify the moderators that the participant wants to join
        self.message_router.conference.serialize_and_send(
            self.participants.connected().moderators().connection_ids(),
            CORE_MODULE_ID,
            None,
            CoreEvent::JoinedWaitingRoom {
                participant_id: participant_origin.id,
                connection_ids: participant.waiting_room_connection_ids().collect(),
                joined_at,
                display_name,
                avatar_url: participant.kind.avatar_url().map(ToString::to_string),
            },
        )?;

        // Notify the participant that they're now in the waiting room
        self.message_router.lobby.serialize_and_send(
            [connection_id],
            CORE_MODULE_ID,
            participant_origin.transaction_id,
            CoreEvent::InWaitingRoom,
        )?;

        tracing::debug!("Participant entered waiting room");

        Ok(())
    }

    fn enter_room(
        &mut self,
        participant_origin: ParticipantOrigin,
        display_name: Option<DisplayName>,
    ) -> Result<(), SignalingModuleError<CoreError>> {
        let Entry::Occupied(mut participant) = self.lobby_participants.entry(participant_origin.id)
        else {
            tracing::debug!("Failed to enter room: participant not known");
            return Err(CoreError::UnknownParticipant.into());
        };

        if !participant.get().accepted {
            tracing::debug!("Failed to enter room: participant not yet accepted");
            return Err(CoreError::NotAccepted.into());
        }

        let conference = self.participants.connected().get(&participant_origin.id);
        let mask_phone_numbers = self.info.mask_unmapped_phone_numbers();
        let default_country_code = self.info.default_country_code();

        let display_name = Self::require_display_name(
            conference,
            participant.get(),
            display_name,
            mask_phone_numbers,
            default_country_code,
        )?;

        let role = participant.get().role;
        let client_kind = participant.get().kind.clone();

        let Entry::Occupied(connection) = participant
            .get_mut()
            .connection_entry(participant_origin.connection_id)
        else {
            tracing::debug!("Failed to enter room: connection not known");
            return Err(CoreError::UnknownConnection.into());
        };

        // If the participant was in the waiting room, notify moderators that they left
        let connections = if connection.get().in_waiting_room {
            let moderator_ids = self.participants.connected().moderators().connection_ids();
            self.message_router.conference.serialize_and_send(
                moderator_ids,
                CORE_MODULE_ID,
                None,
                CoreEvent::LeftWaitingRoom(LeftWaitingRoom {
                    id: participant_origin.id,
                    connection_ids: participant.get().waiting_room_connection_ids().collect(),
                }),
            )?;

            // When coming from the waiting room, all waiting connections enter the room together...
            self.message_router
                .upgrade_connections(participant.get().waiting_room_connection_ids());
            participant.get_mut().drain_waiting_connections().collect()
        } else {
            // ...otherwise only the single connection that requested to enter does.
            let (
                connection_id,
                LobbyConnection {
                    device_id,
                    in_waiting_room: _,
                },
            ) = connection.remove_entry();
            self.message_router
                .upgrade_connections([connection_id].into_iter());
            vec![(connection_id, device_id)]
        };

        if !participant.get().is_connected() {
            participant.remove();
        }

        for (connection_id, device_id) in connections {
            self.join_room(
                participant_origin.id,
                connection_id,
                device_id,
                client_kind.clone(),
                display_name.clone(),
                role,
            )?;
        }

        tracing::debug!("Participant entered room");
        Ok(())
    }

    /// Mask the display name of call-in participants when the corresponding default is enabled.
    ///
    /// Call-in participants typically use the raw phone number they dialled in as their display
    /// name. This wrapper replaces it with a masked variant (see
    /// [`super::call_in::mask_display_name`]) so peers never see the raw number.
    fn mask_call_in_display_name(
        display_name: DisplayName,
        mask_phone_numbers: Option<bool>,
        default_country_code: Option<&str>,
    ) -> DisplayName {
        let mask_phone_numbers = mask_phone_numbers.unwrap_or(true);
        if !mask_phone_numbers {
            return display_name;
        }

        let parsed_country_code = default_country_code
            .map(phonenumber::country::Id::from_str)
            .and_then(Result::ok);
        if default_country_code.is_some() && parsed_country_code.is_none() {
            tracing::warn!(
                "could not parse configured default country code, which is likely due to a misconfigured `[call-in]` section in the controller"
            );
        }

        super::call_in::mask_display_name(display_name, parsed_country_code)
    }

    /// Derives the display name for a participant based on their current state in the conference
    /// and lobby.
    pub fn derive_display_name(
        conference: Option<&ParticipantState>,
        lobby: Option<&LobbyParticipant>,
    ) -> Option<DisplayName> {
        // TODO: if we want to prevent registered users from changing their display name, we need to
        // fall back to the name from their profile here
        conference
            .as_ref()
            .map(|state| state.display_name.clone())
            .or_else(|| {
                lobby.and_then(|state| {
                    state
                        .waiting_room_state
                        .as_ref()
                        .map(|state| state.display_name.clone())
                })
            })
    }

    /// Determine the participant's display name based on the provided display name and the
    /// participant's state in the conference and lobby.
    fn require_display_name(
        conference: Option<&ParticipantState>,
        lobby: &LobbyParticipant,
        provided: Option<DisplayName>,
        mask_phone_numbers: Option<bool>,
        default_country_code: Option<&str>,
    ) -> Result<DisplayName, CoreError> {
        // Derive the display name in the following order of precedence:
        // 1. Conference state (when the participant already has a connection in the conference)
        // 2. Waiting room state (when the participant already has a waiting connection)
        // 3. Value provided with the command
        // 4. Public user profile (registered users) or a fixed default (services)
        let derived = Self::derive_display_name(conference, Some(lobby));

        if let (Some(provided_name), Some(derived_name)) = (&provided, &derived)
            && provided_name != derived_name
        {
            return Err(CoreError::DisplayNameAlreadyProvided);
        }

        let display_name = provided.or(derived);
        let client_kind = &lobby.kind;

        let Some(display_name) = display_name else {
            // No display name was provided or derived:
            // - If the participant is a registered user return the user info display name
            // - If the participant is a service return the default service name
            // - If the participant is a guest return display name required error
            return match client_kind {
                ClientKind::Registered { profile } => Ok(profile.user_info.display_name.clone()),
                ClientKind::Recorder { .. } => {
                    let display_name = DisplayName::from_str_lossy(DEFAULT_RECORDER_NAME);
                    Ok(display_name)
                }
                ClientKind::Transcription { .. } => {
                    let display_name = DisplayName::from_str_lossy(DEFAULT_TRANSCRIPTION_NAME);
                    Ok(display_name)
                }
                ClientKind::CallIn | ClientKind::Guest => {
                    return Err(CoreError::DisplayNameRequired);
                }
            };
        };

        if client_kind.is_call_in() {
            return Ok(Self::mask_call_in_display_name(
                display_name,
                mask_phone_numbers,
                default_country_code,
            ));
        }

        Ok(display_name)
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
        let event = SignalingEvent {
            namespace: CORE_MODULE_ID,
            transaction_id: None,
            timestamp: Timestamp::now(),
            payload: CoreEvent::JoinBlocked {
                reason: JoinBlockedReason::ParticipantLimitReached,
            },
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
                    &mut self.lobby_participants,
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
            &mut self.lobby_participants,
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

    fn meeting_details(&self) -> MeetingDetails {
        MeetingDetails {
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
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn build_join_success(
        &self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
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
            .map(|state| state.display_name.clone())
            .ok_or_else(|| FatalError(anyhow!("Participant state not found")))?;
        let (role, avatar_url, is_room_owner) = match client_kind {
            ClientKind::Registered { profile } => (
                role.to_opentalk_types_signaling_role(),
                Some(profile.user_info.avatar_url),
                self.info.room.created_by.id == profile.id,
            ),
            ClientKind::Guest
            | ClientKind::Transcription { .. }
            | ClientKind::Recorder { .. }
            | ClientKind::CallIn => (Role::Guest, None, false),
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

        // Moderators can always see meeting details
        let meeting_details = (self.info.room.show_meeting_details || role == Role::Moderator)
            .then(|| self.meeting_details());

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
                alias: self.info.room.alias.clone(),
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
            display_name: joinee.display_name.clone(),
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
            display_name: state.display_name.clone(),
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
            .lobby
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
    use std::{
        assert_matches,
        collections::{BTreeMap, BTreeSet, HashMap},
    };

    use chrono::DateTime;
    use insta::assert_snapshot;
    use opentalk_roomserver_signaling::{
        lobby_participant::{LobbyParticipant, WaitingRoomState},
        participant_state::ParticipantState,
        signaling_event::SignalingEvent,
    };
    use opentalk_roomserver_types::{
        client_parameters::{ClientKind, Role as ClientRole},
        connection_id::ConnectionId,
        core::{CORE_MODULE_ID, CoreError},
        device_id::DeviceId,
        join::{connection_info::ConnectionInfo, join_success::JoinSuccess},
        public_user_profile::PublicUserProfile,
        room_info::RoomInfo,
        room_kind::RoomKind,
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
    use opentalk_types_signaling::{ModuleData, Role, SignalingModuleFrontendData};
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use super::{CoreEvent, RoomTask};
    use crate::mocking::socket::MockSocket;

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
                alias: None,
                password: None,
                created_by: UserInfo::example_data(),
            },
            meeting_details: Some(MeetingDetails {
                call_in: None,
                streaming_links: vec![],
            }),
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

    /// Builds a lobby participant of the given kind, optionally with a name set in the waiting
    /// room.
    fn lobby_participant(kind: ClientKind, waiting_room_name: Option<&str>) -> LobbyParticipant {
        LobbyParticipant::new(
            kind,
            ClientRole::User,
            HashMap::new(),
            false,
            waiting_room_name.map(|name| WaitingRoomState {
                display_name: DisplayName::from_str_lossy(name),
                joined_at: DateTime::UNIX_EPOCH,
            }),
        )
    }

    /// Builds the conference state of a connected participant with the given display name.
    fn participant_state(kind: ClientKind, name: &str) -> ParticipantState {
        ParticipantState::new(
            kind,
            DisplayName::from_str_lossy(name),
            ClientRole::User,
            DateTime::UNIX_EPOCH,
            false,
        )
    }

    #[test]
    fn require_display_name_uses_the_conference_name_when_none_is_provided() {
        let conference = participant_state(ClientKind::Guest, "Conference Name");
        let lobby = lobby_participant(ClientKind::Guest, None);

        let name = RoomTask::<MockSocket>::require_display_name(
            Some(&conference),
            &lobby,
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(name, DisplayName::from_str_lossy("Conference Name"));
    }

    #[test]
    fn require_display_name_rejects_a_name_provided_while_in_the_conference() {
        let conference = participant_state(ClientKind::Guest, "Conference Name");
        let lobby = lobby_participant(ClientKind::Guest, None);

        let err = RoomTask::<MockSocket>::require_display_name(
            Some(&conference),
            &lobby,
            Some(DisplayName::from_str_lossy("Other Name")),
            None,
            None,
        )
        .unwrap_err();

        assert_matches!(err, CoreError::DisplayNameAlreadyProvided);
    }

    #[test]
    fn require_display_name_prefers_conference_name_over_waiting_room_name() {
        let conference = participant_state(ClientKind::Guest, "Conference Name");
        let lobby = lobby_participant(ClientKind::Guest, Some("Waiting Room Name"));

        let name = RoomTask::<MockSocket>::require_display_name(
            Some(&conference),
            &lobby,
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(name, DisplayName::from_str_lossy("Conference Name"));
    }

    #[test]
    fn require_display_name_uses_waiting_room_name_when_none_is_provided() {
        let lobby = lobby_participant(ClientKind::Guest, Some("Waiting Room Name"));

        let name =
            RoomTask::<MockSocket>::require_display_name(None, &lobby, None, None, None).unwrap();

        assert_eq!(name, DisplayName::from_str_lossy("Waiting Room Name"));
    }

    #[test]
    fn require_display_name_rejects_a_name_provided_while_in_the_waiting_room() {
        let lobby = lobby_participant(ClientKind::Guest, Some("Waiting Room Name"));

        let err = RoomTask::<MockSocket>::require_display_name(
            None,
            &lobby,
            Some(DisplayName::from_str_lossy("Other Name")),
            None,
            None,
        )
        .unwrap_err();

        assert_matches!(err, CoreError::DisplayNameAlreadyProvided);
    }

    #[test]
    fn require_display_name_requires_a_name_from_guests() {
        let lobby = lobby_participant(ClientKind::Guest, None);

        let err = RoomTask::<MockSocket>::require_display_name(None, &lobby, None, None, None)
            .unwrap_err();

        assert_matches!(err, CoreError::DisplayNameRequired);
    }

    #[test]
    fn require_display_name_accepts_a_name_from_guests() {
        let lobby = lobby_participant(ClientKind::Guest, None);

        let name = RoomTask::<MockSocket>::require_display_name(
            None,
            &lobby,
            Some(DisplayName::from_str_lossy("Gustav")),
            None,
            None,
        )
        .unwrap();

        assert_eq!(name, DisplayName::from_str_lossy("Gustav"));
    }

    #[test]
    fn require_display_name_requires_a_name_from_call_in_participants() {
        let lobby = lobby_participant(ClientKind::CallIn, None);

        let err = RoomTask::<MockSocket>::require_display_name(None, &lobby, None, None, None)
            .unwrap_err();

        assert_matches!(err, CoreError::DisplayNameRequired);
    }

    #[test]
    fn require_display_name_accepts_a_name_from_call_in_participants() {
        let lobby = lobby_participant(ClientKind::CallIn, None);

        let name = RoomTask::<MockSocket>::require_display_name(
            None,
            &lobby,
            Some(DisplayName::from_str_lossy("1234")),
            Some(false), // disable phone number masking
            None,
        )
        .unwrap();

        assert_eq!(name, DisplayName::from_str_lossy("1234"));
    }

    #[test]
    fn require_display_name_falls_back_to_the_profile_name_for_registered_users() {
        let profile = PublicUserProfile::example_data();
        let expected = profile.user_info.display_name.clone();
        let lobby = lobby_participant(ClientKind::Registered { profile }, None);

        let name =
            RoomTask::<MockSocket>::require_display_name(None, &lobby, None, None, None).unwrap();

        assert_eq!(name, expected);
    }

    #[test]
    fn require_display_name_lets_registered_users_override_their_profile_name() {
        let lobby = lobby_participant(
            ClientKind::Registered {
                profile: PublicUserProfile::example_data(),
            },
            None,
        );

        let name = RoomTask::<MockSocket>::require_display_name(
            None,
            &lobby,
            Some(DisplayName::from_str_lossy("Custom Name")),
            None,
            None,
        )
        .unwrap();

        assert_eq!(name, DisplayName::from_str_lossy("Custom Name"));
    }

    #[test]
    fn require_display_name_uses_a_fixed_default_for_recorders() {
        let lobby = lobby_participant(
            ClientKind::Recorder {
                room: RoomKind::Main,
            },
            None,
        );

        let name =
            RoomTask::<MockSocket>::require_display_name(None, &lobby, None, None, None).unwrap();

        assert_eq!(name, DisplayName::from_str_lossy("recorder"));
    }

    #[test]
    fn require_display_name_uses_a_fixed_default_for_transcriptions() {
        let lobby = lobby_participant(
            ClientKind::Transcription {
                room: RoomKind::Main,
            },
            None,
        );

        let name =
            RoomTask::<MockSocket>::require_display_name(None, &lobby, None, None, None).unwrap();

        assert_eq!(name, DisplayName::from_str_lossy("transcription"));
    }

    #[test]
    fn derive_display_name_returns_none_for_guests_without_prior_presence() {
        let lobby = lobby_participant(ClientKind::Guest, None);

        let name = RoomTask::<MockSocket>::derive_display_name(None, Some(&lobby));

        assert_eq!(name, None);
    }

    #[test]
    fn derive_display_name_does_not_fall_back_to_the_profile_name_for_registered_users() {
        let lobby = lobby_participant(
            ClientKind::Registered {
                profile: PublicUserProfile::example_data(),
            },
            None,
        );

        let name = RoomTask::<MockSocket>::derive_display_name(None, Some(&lobby));

        assert_eq!(name, None);
    }

    #[test]
    fn derive_display_name_uses_the_waiting_room_name() {
        let lobby = lobby_participant(ClientKind::Guest, Some("Waiting Room Name"));

        let name = RoomTask::<MockSocket>::derive_display_name(None, Some(&lobby));

        assert_eq!(name, Some(DisplayName::from_str_lossy("Waiting Room Name")));
    }

    #[test]
    fn derive_display_name_prefers_the_conference_name_over_the_waiting_room_name() {
        let conference = participant_state(ClientKind::Guest, "Conference Name");
        let lobby = lobby_participant(ClientKind::Guest, Some("Waiting Room Name"));

        let name = RoomTask::<MockSocket>::derive_display_name(Some(&conference), Some(&lobby));

        assert_eq!(name, Some(DisplayName::from_str_lossy("Conference Name")));
    }
}

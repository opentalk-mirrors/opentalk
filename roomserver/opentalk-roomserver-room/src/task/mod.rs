// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! The [`RoomTask`] is the central component of a meeting room. It initializes
//! the [`MessageRouter`] and [`SignalingModule`]s. The [`RoomTask`] also
//! * accepts new participants and adds them to the [`MessageRouter`]
//! * forwards messages from the [`MessageRouter`] to the [`SignalingModule`]s
//! * forwards [`LoopbackMessage`]s between the [`SignalingModule`]s
//!
//! ```text
//! ┌──────────┐
//! │ RoomTask │
//! └──┬─┬─────┘
//!    │ └──────────────┐
//!    │                │
//! ┌──▼────────────┐ ┌─▼───────┐
//! │ MessageRouter │ │ Modules │
//! └───────────────┘ └─┬───────┘
//!                     │
//!                   ┌─▼────────────┐
//!                   │   <Trait>    │
//!                   │ ModuleHandle │
//!                   └─▲ ───────────┘
//!                     │
//!                    Implements
//!                     │
//!                   ┌─┴────────────────┐
//!                   │ ModuleDispatcher │
//!                   └─┬────────────────┘
//!                     │
//!                   ┌─▼───────────────┐
//!                   │     <Trait>     │
//!                   │ SignalingModule │
//!                   └─────────────────┘
//! ```
//!
//! # [`ConnectionId`] and [`ParticipantId`]
//!
//! Every connection to a Room is identified by the [`ConnectionId`]. The connection ID is generated
//! by [`MessageRouter::add_lobby_connection`].
//!
//! For registered users, the [`ParticipantId`] is derived from the [`UserId`] that is part of the
//! [`PublicUserProfile`]. Guests and services don't have a such a profile. These clients provide a
//! `device_secret` that is used to derive a [`DeviceId`] which in turn is used to derive the
//! [`ParticipantId`].
//!
//! [`SignalingModule`]: opentalk_roomserver_signaling::signaling_module::SignalingModule
//! [`UserId`]: opentalk_types_common::users::UserId
//! [`PublicUserProfile`]: opentalk_roomserver_types::public_user_profile::PublicUserProfile

use std::{
    cell::RefCell,
    collections::{
        HashMap,
        hash_map::Entry::{Occupied, Vacant},
    },
    mem,
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use breakout::state::BreakoutState;
use chrono::Utc;
use futures::stream::{FuturesUnordered, StreamExt};
use opentalk_roomserver_signaling::{
    banned_participant::BannedParticipant,
    event_origin::{EventOrigin, ParticipantOrigin},
    instruction::Instruction,
    internal_module_message::InterModuleMessage,
    lobby_participant::LobbyParticipant,
    loopback::{LoopbackFuture, LoopbackMessage},
    module_context::ModuleMessage,
    participant_state::{ParticipantState, Participants},
    room_info::RoomTaskInfo,
    signaling_module::SignalingModuleInitData,
};
use opentalk_roomserver_types::{
    breakout::BREAKOUT_MODULE_ID,
    client_allowed::{ClientAllowed, ClientNotAllowedReason},
    client_parameters::{ClientKind, ClientParameters, ParticipationKind, Role},
    connection_id::ConnectionId,
    core::{CORE_MODULE_ID, RoomCloseReason},
    device_id::DeviceId,
    error::SignalingError,
    kick_reason::KickReason,
    room_kind::RoomKind,
    room_parameters::RoomParameters,
    signaling::{SignalingCommand, module_error::FatalError, websocket::SignalingSocket},
};
use opentalk_types_api_internal::module_assets::Quota;
use opentalk_types_common::{
    modules::ModuleId,
    rooms::RoomId,
    roomserver::DeviceSecret,
    tariffs::QuotaType,
    time::Timestamp,
    users::{DisplayName, UserId},
};
use opentalk_types_signaling::ParticipantId;
use tokio::sync::{mpsc, oneshot};
use tracing::Instrument;
use uuid::Uuid;

use super::{message_router::CloseReason, signaling::module_initializer::Modules};
use crate::{
    message_router::{MessageEnvelope, MessageRouter, ScopedRouter, SignalingMessage},
    metrics::Metrics,
    signaling::{DynBroadcastEvent, DynEvent, dyn_module_context::DynModuleContext},
    task::{
        context::RoomTaskContext,
        handle::{Request, RoomTaskHandle, TaskMessage},
        timeout::Timeout,
    },
};

pub mod breakout;
pub mod closing;
pub mod context;
pub mod core;
pub mod handle;
mod livekit;
pub mod lobby;
pub mod timeout;

mod call_in;

#[derive(Debug, thiserror::Error)]
pub enum RoomTaskApiError {
    /// Placeholder error for features that are currently missing.
    #[error("This functionality is currently not available")]
    NotImplemented,

    #[error("The patch could not be applied")]
    FailedToApplyPatch(anyhow::Error),

    #[error("The specified resource could not be found")]
    NotFound,

    #[error("Access denied")]
    Unauthorized,

    #[error("There was an unexpected error")]
    Internal,

    /// The room task is shutting down and cannot process the request.
    #[error("The room task is shutting down and cannot process the request")]
    Closing,
}

/// The [`RoomTask`] manages the conference state and signaling.
///
/// An idle [`Timeout`] starts when a room has no participants in it. When the idle timeout is
/// reached, the room task exits.
pub struct RoomTask<Socket: SignalingSocket + 'static> {
    info: RoomTaskInfo,

    /// The receiver for web server API request that target this room
    api_rx: mpsc::Receiver<TaskMessage<Socket>>,

    /// The rooms idle timeout, only active when no participants are in the room.
    idle_timeout: Timeout,

    message_router: MessageRouter,

    breakout_config: Option<BreakoutState>,

    /// Loopback futures that were created by signaling modules
    loopback_futures: FuturesUnordered<LoopbackFuture>,
    /// Cancellation sender for the loopback futures. When dropped or sent, the loopback futures
    /// will return `None` when all futures are completed.
    loopback_cancel_tx: Option<oneshot::Sender<()>>,

    participants: Participants,

    modules: Modules,

    ctx: RoomTaskContext,

    /// Collection of participants in the lobby.
    lobby_participants: HashMap<ParticipantId, LobbyParticipant>,

    /// Set of participants that are banned from the room
    banned_participants: HashMap<ParticipantId, BannedParticipant>,

    /// Timeout for the room time limit quota
    quota_timeout: Timeout,

    metrics: Metrics,
}

impl<Socket: SignalingSocket> RoomTask<Socket> {
    /// Spawns a new [`RoomTask`] with a specific timeout
    #[tracing::instrument(level = "info", skip_all, fields(opentalk.room_id = %room_id))]
    pub fn setup(
        ctx: RoomTaskContext,
        room_id: RoomId,
        mut room_parameters: Arc<RoomParameters>,
    ) -> (RoomTaskHandle<Socket>, impl Future<Output = ()> + Send) {
        let (tx, rx) = mpsc::channel(20);

        let message_router = MessageRouter::new(
            room_id,
            ctx.app_state.clone(),
            room_parameters.ws_rate_limit,
        );
        let room_handle = RoomTaskHandle {
            assets: Arc::clone(&ctx.asset_storage),
            module_resources: Arc::clone(&ctx.module_resources),
            sender: tx,
        };

        let future_room = Box::pin(async move {
            let (modules, uninitialized) = ctx
                .module_registry
                .initialize_modules(SignalingModuleInitData {
                    settings: Arc::clone(&ctx.settings),
                    room_parameters: Arc::clone(&room_parameters),
                })
                .await;

            if !uninitialized.is_empty() {
                // Remove unknown modules from the room parameters
                let mut params = (*room_parameters).clone();
                for module_id in uninitialized {
                    tracing::debug!("Unable to initialize module {module_id} for room {room_id}");
                    params.module_settings.remove(&module_id);
                }
                room_parameters = Arc::new(params);
            }

            let room_info = RoomTaskInfo {
                room_id,
                closes_at: room_parameters.calc_time_limit_quota(Timestamp::now()),
                room: (*room_parameters).clone(),
            };

            let (loopback_cancel_tx, loopback_rx) = oneshot::channel::<()>();
            let loopback_futures: FuturesUnordered<LoopbackFuture> = FuturesUnordered::new();
            loopback_futures.push(Box::pin(async {
                let _ = loopback_rx.await;
                tracing::debug!("Loopback guard canceled, loopback futures may finish");
                None
            }));

            let time_limit = room_info
                .room
                .tariff
                .quota(&QuotaType::RoomTimeLimitSecs)
                .unwrap_or(0);

            let room_task = RoomTask {
                ctx,
                info: room_info,
                api_rx: rx,
                idle_timeout: Timeout::start_new(room_parameters.room_idle_timeout),
                message_router,
                breakout_config: None,
                loopback_cancel_tx: Some(loopback_cancel_tx),
                loopback_futures,
                participants: Participants::new(),
                modules,
                lobby_participants: HashMap::new(),
                banned_participants: HashMap::new(),
                quota_timeout: Timeout::new(Duration::from_secs(time_limit)),
                metrics: Metrics::new(),
            };

            room_task.run().await;
            tracing::debug!("RoomTask closed");
        })
        .instrument(tracing::Span::current());

        (room_handle, future_room)
    }

    async fn run(mut self) {
        tracing::debug!("Spawn room with modules: {:?}", self.modules.keys());
        let room_id = self.info.room_id;

        let close_reason = self
            .inner_run()
            .await
            .inspect_err(|e| {
                tracing::error!("RoomTask exited with error {e:?}");
            })
            .unwrap_or(RoomCloseReason::FatalError);

        tracing::debug!("Close room {room_id}");

        if let Err(e) = self.close(close_reason).await {
            tracing::error!("RoomTask closing loop exited with error {e:?}");
        }
    }

    async fn inner_run(&mut self) -> Result<RoomCloseReason, FatalError> {
        loop {
            tokio::select! {
                msg = self.api_rx.recv() => {
                    let Some(msg) = msg else {
                        // TaskHandle dropped, exiting
                        tracing::warn!("Room tasks {} api channel was dropped, exiting", self.info.room_id);
                        return Ok(RoomCloseReason::ImmediateShutdown);
                    };

                    match self.handle_api_request(msg).await {
                        Ok(None) => {},
                        Ok(Some(reason)) => return Ok(reason),
                        Err(e) => tracing::error!("Failed to handle room task api request: {e:?}")
                    }
                },
                msg = self.message_router.recv() => {
                    self.handle_message(msg)?;
                }
                Some(msg) = self.loopback_futures.next() => {
                    self.handle_loopback(msg)?;
                },
                () = self.idle_timeout.wait_for_completion() => {
                    tracing::debug!("Room task {} reached its idle timeout, exiting", self.info.room_id);
                    return Ok(RoomCloseReason::IdleTimeoutReached);
                }
                () = Self::check_breakout_timeout(&mut self.breakout_config) => {
                    self.breakout_expired();
                }
                result = self.ctx.app_state.changed() => {
                    if result.is_err() || self.ctx.app_state.borrow().is_shutting_down() {
                        tracing::debug!("Room task {} received shutdown signal, exiting", self.info.room_id);
                        return Ok(RoomCloseReason::GracefulShutdown)
                    }
                }
                () = self.quota_timeout.wait_for_completion() => {
                    tracing::debug!("Room task {} reached its time limit, exiting", self.info.room_id);
                    return Ok(RoomCloseReason::TimeLimitReached);
                }
            };
            for (connection_id, participant_id) in self
                .message_router
                .conference_disconnected()
                .collect::<Vec<_>>()
            {
                self.disconnect_participant(
                    EventOrigin::Internal,
                    participant_id,
                    connection_id,
                    CloseReason::ConnectionLost,
                )?;
            }
            for (connection_id, participant_id) in
                self.message_router.lobby_disconnected().collect::<Vec<_>>()
            {
                self.disconnect_lobby_participant(participant_id, connection_id);
            }
        }
    }

    #[tracing::instrument(skip_all, parent = &msg.span, fields(opentalk.room_id = %self.info.room_id))]
    async fn handle_api_request(
        &mut self,
        msg: TaskMessage<Socket>,
    ) -> anyhow::Result<Option<RoomCloseReason>> {
        match msg.request {
            Request::RefreshIdleTimeout { response } => {
                self.refresh_idle_timeout();
                response
                    .send(Ok(()))
                    .ok()
                    .context("Failed to respond to RefreshIdleTimeout, response channel dropped")?;
            }
            Request::SetParameters {
                response: response_tx,
                parameters,
            } => {
                self.set_parameters(*parameters);
                response_tx
                    .send(Ok(()))
                    .ok()
                    .context("Failed to respond to UpdateParameter, response channel dropped")?;
            }
            Request::PatchParameters {
                response: response_tx,
                patch,
            } => {
                if let Err(err) = patch.clone().try_apply(&mut self.info.room) {
                    response_tx
                        .send(Err(RoomTaskApiError::FailedToApplyPatch(err)))
                        .ok()
                        .context("Failed to respond to PatchParameter, response channel dropped")?;

                    return Ok(None);
                }

                self.broadcast_room_parameters_changed_event(*patch)
                    .context("Failed to broadcast updated room parameters to participants")?;

                response_tx
                    .send(Ok(()))
                    .ok()
                    .context("Failed to respond to PatchParameter, response channel dropped")?;
            }
            Request::StorageQuotaChanged {
                response: response_tx,
                quota,
            } => {
                self.storage_quota_changed(quota)?;

                response_tx.send(Ok(())).ok().context(
                    "Failed to respond to StorageQuotaChanged, response channel dropped",
                )?;
            }
            Request::IsClientAllowed {
                response: response_tx,
                user_id,
                participation_kind,
            } => {
                let client_allowed = self.is_client_allowed(user_id, participation_kind);

                response_tx
                    .send(Ok(client_allowed))
                    .ok()
                    .context("Failed to respond to IsClientAllowed, response channel dropped")?;
            }
            Request::AllowedOrigins { response } => response
                .send(Ok(self.info.room.allowed_origins.clone()))
                .ok()
                .context("Failed to respond to AllowedOrigins, response channel dropped")?,
            Request::WsJoin {
                response,
                socket,
                client_parameters,
            } => {
                self.ws_join(socket, client_parameters)
                    .context("Fatal error while accepting new websocket connection")?;
                response
                    .send(Ok(()))
                    .ok()
                    .context("Failed to respond to WsJoin, response channel dropped")?;
            }
            Request::ConnectUpstreamLivekitSocket {
                response,
                websocket_request,
            } => {
                self.connect_upstream_socket(websocket_request, response)?;
            }
            Request::ConnectDownstreamLivekitSocket {
                response,
                websocket_request,
                upstream_socket,
                downstream_socket,
            } => {
                self.connect_downstream_socket(
                    websocket_request,
                    *upstream_socket,
                    downstream_socket,
                    response,
                )?;
            }
            Request::GetLivekitServiceUrl { response } => {
                self.get_livekit_service_url(response)?;
            }
            Request::Close { response, reason } => {
                // Do not return an error when sending the response fails because that would mean
                // the room tasks keeps running until terminated with the `ImmediateShutdown` reason
                // due to being dropped.
                response.send(Ok(())).ok().unwrap_or_else(|| {
                    tracing::error!("Failed to respond to Close, response channel dropped")
                });

                return Ok(Some(reason));
            }
        }

        Ok(None)
    }

    fn handle_loopback(&mut self, msg: Option<LoopbackMessage>) -> Result<(), FatalError> {
        let Some(msg) = msg else {
            tracing::debug!("Optional loopback future returned None");
            return Ok(());
        };

        let span = msg.span.clone();
        let _enter = span.enter();
        self.handle_loopback_message(msg)
    }

    fn handle_loopback_message(&mut self, msg: LoopbackMessage) -> Result<(), FatalError> {
        let Some(module) = self.modules.get_mut(&msg.namespace) else {
            tracing::error!(
                "Received loopback event for unknown module {}",
                msg.namespace
            );
            return Ok(());
        };
        let mut messages = RefCell::new(Vec::new());
        let transaction_id = msg.origin.transaction_id();
        let mut ctx = DynModuleContext::new(
            self.info.room_id,
            msg.room,
            msg.origin,
            &mut self.info,
            &mut self.participants,
            &mut self.lobby_participants,
            &mut self.banned_participants,
            msg.timestamp,
            Arc::clone(&self.ctx.asset_storage),
            Arc::clone(&self.ctx.module_resources),
            &mut messages,
            &mut self.loopback_futures,
        );

        let room = ctx.room;
        let origin = ctx.event_origin;
        let timestamp = ctx.timestamp;

        if let Err(err) = module.on_event(&mut ctx, DynEvent::LoopbackEvent(msg.value)) {
            self.handle_fatal_module_error(msg.namespace, transaction_id, err);
        }

        self.handle_module_messages(messages, room, origin, timestamp)
    }

    fn handle_module_messages(
        &mut self,
        messages: RefCell<Vec<ModuleMessage>>,
        room_kind: RoomKind,
        origin: EventOrigin,
        timestamp: Timestamp,
    ) -> Result<(), FatalError> {
        for message in messages.into_inner() {
            match message {
                ModuleMessage::Websocket {
                    connection_id,
                    message,
                } => {
                    self.message_router
                        .conference
                        .send_event([connection_id], message);
                }
                ModuleMessage::WaitingRoomWebsocket {
                    connection_id,
                    message,
                } => self
                    .message_router
                    .lobby
                    .send_event([connection_id], message),
                ModuleMessage::InternalCommand(inter_module_message) => {
                    self.handle_internal_command(
                        inter_module_message,
                        room_kind,
                        origin,
                        timestamp,
                    )?;
                }
                ModuleMessage::Instruction(instruction) => {
                    self.handle_instruction(origin, instruction)?;
                }
            }
        }
        Ok(())
    }

    fn handle_internal_command(
        &mut self,
        command: InterModuleMessage,
        room: RoomKind,
        origin: EventOrigin,
        timestamp: Timestamp,
    ) -> Result<(), FatalError> {
        let Some(module) = self.modules.get_mut(&command.receiver) else {
            tracing::error!(
                "Received internal command for unknown module '{}' from module '{}'",
                command.receiver,
                command.sender,
            );
            return Ok(());
        };
        tracing::debug!(
            "Handling internal command from module '{}' to module '{}'",
            command.sender,
            command.receiver
        );

        let mut messages = RefCell::new(Vec::new());
        let mut ctx = DynModuleContext::new(
            self.info.room_id,
            room,
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
        );

        if let Err(err) = module.on_event(
            &mut ctx,
            DynEvent::InternalCommand {
                sender: command.sender,
                command: command.command,
            },
        ) {
            self.handle_fatal_module_error(command.receiver, origin.transaction_id(), err);
        }

        // Disallow modules to trigger internal commands from internal commands
        messages.borrow_mut().retain(|message| match message {
            ModuleMessage::InternalCommand(InterModuleMessage {
                sender, receiver, ..
            }) => {
                tracing::warn!(
                    "Dropping internal command from '{sender}' to '{receiver}' to prevent recursion"
                );
                false
            }
            _ => true,
        });

        self.handle_module_messages(messages, room, origin, timestamp)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn handle_instruction(
        &mut self,
        origin: EventOrigin,
        instruction: Instruction,
    ) -> Result<(), FatalError> {
        match instruction {
            Instruction::Kick {
                participants,
                reason,
            } => self.kick_participants(origin, participants, reason),
            Instruction::Ban { participant } => self.ban_participants(origin, participant),
            Instruction::MoveToWaitingRoom { participant } => {
                self.move_to_waiting_room(participant)
            }
            Instruction::BroadcastModerationRoleChange { participant } => {
                self.broadcast_moderation_role_change(origin, participant)
            }
        }
    }

    fn calculate_new_role(role: Role, reason: &KickReason) -> Role {
        match role {
            Role::User => Role::User,
            // In case of debriefing, moderators must keep their roles
            // Otherwise degrade them to users
            Role::Moderator => match *reason {
                KickReason::Kicked | KickReason::GuestAccessDisabled => Role::User,
                KickReason::Debriefed => Role::Moderator,
            },
        }
    }

    fn kick_participants(
        &mut self,
        origin: EventOrigin,
        participants: Vec<ParticipantId>,
        reason: KickReason,
    ) -> Result<(), FatalError> {
        let close_reason = CloseReason::from(reason.clone());
        for participant_id in participants {
            let mut lobby_connections = Vec::new();
            let mut conference_connections = Vec::new();

            if let Some(state) = self.lobby_participants.get_mut(&participant_id) {
                state.accepted = false;
                state.role = Self::calculate_new_role(state.role, &reason);
                lobby_connections.extend(state.all_connection_ids());
            }

            if let Some(state) = self.participants.all_unfiltered.get_mut(&participant_id) {
                // Ensure the kicked participant can not skip the waiting room
                state.in_waiting_room = true;
                state.role = Self::calculate_new_role(state.role, &reason);
                conference_connections.extend(state.connections());
            }

            if lobby_connections.is_empty() && conference_connections.is_empty() {
                tracing::error!(
                    "Failed to get connections for unknown participant {participant_id}"
                );
                continue;
            }

            // Disconnect the lobby connections first, so they don't receive events produced by the
            // conference connections disconnecting (e.g. `CoreEvent::DisplayNameAssigned`).
            for connection_id in lobby_connections {
                self.disconnect_lobby_participant(participant_id, connection_id);
            }

            for connection_id in conference_connections {
                self.disconnect_participant(origin, participant_id, connection_id, close_reason)?;
            }
        }

        Ok(())
    }

    fn ban_participants(
        &mut self,
        origin: EventOrigin,
        participant_id: ParticipantId,
    ) -> Result<(), FatalError> {
        let mut lobby_connections = Vec::new();
        let mut conference_connections = Vec::new();

        if let Some(state) = self.lobby_participants.get(&participant_id) {
            lobby_connections.extend(state.all_connection_ids());
        }

        if let Some(state) = self.participants.all_unfiltered.get(&participant_id) {
            conference_connections.extend(state.connections());
        }

        if lobby_connections.is_empty() && conference_connections.is_empty() {
            tracing::error!("Failed to ban participant, {participant_id} does not exist");
            return Ok(());
        };

        // Disconnect the lobby connections first, so they don't receive events produced by the
        // conference connections disconnecting (e.g. `CoreEvent::DisplayNameAssigned`).
        for connection_id in lobby_connections {
            self.disconnect_lobby_participant(participant_id, connection_id);
        }

        for connection_id in conference_connections {
            self.disconnect_participant(
                origin,
                participant_id,
                connection_id,
                CloseReason::Banned,
            )?;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub fn broadcast_moderation_role_change(
        &mut self,
        origin: EventOrigin,
        participant_id: ParticipantId,
    ) -> Result<(), FatalError> {
        let Some(room) = self
            .participants
            .all_unfiltered
            .get(&participant_id)
            .map(|participant| participant.room)
        else {
            tracing::error!("Role changed for unknown participant");
            return Ok(());
        };

        self.broadcast_event_to_modules(
            origin,
            room,
            DynBroadcastEvent::ModerationRoleChanged { participant_id },
        )
        .handle_requested_messages(self)
    }

    #[tracing::instrument(level = "info", skip_all)]
    fn refresh_idle_timeout(&mut self) {
        self.idle_timeout.reset();
    }

    #[tracing::instrument(level = "info", skip(self))]
    fn set_parameters(&mut self, room_parameters: RoomParameters) {
        self.info.room = room_parameters
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn is_client_allowed(&self, user_id: Option<UserId>, kind: ParticipationKind) -> ClientAllowed {
        // Check if the user is banned
        if let Some(user_id) = user_id {
            let participant_id = ParticipantId::from(Uuid::from(user_id));
            if self.banned_participants.contains_key(&participant_id) {
                return ClientAllowed::No {
                    reason: ClientNotAllowedReason::Banned,
                };
            }
        }

        // Check services before guest access, so services receive the correct `e2ee_enabled` reason
        // instead of the `not_found` that is sent to guests.
        if self.info.room.e2e_encryption && kind.is_service_or_callin() {
            return ClientAllowed::No {
                reason: ClientNotAllowedReason::E2eeEnabled,
            };
        }

        // Check guest access
        if !self.info.room.guest_access && kind.is_guest_or_callin() {
            ClientAllowed::No {
                reason: ClientNotAllowedReason::GuestAccessDisabled,
            }
        } else {
            ClientAllowed::Yes
        }
    }

    #[tracing::instrument(level = "info", skip_all)]
    fn storage_quota_changed(&mut self, quota: Quota) -> Result<(), FatalError> {
        // Update the quota values in the room parameters, so joining participants receive the new
        // values in the JoinSuccess event.
        let tariff = &mut self.info.room.tariff;
        if let Some(total) = quota.total {
            tariff.quotas.insert(QuotaType::MaxStorage, total);
        }
        tariff.used_quota.insert(QuotaType::MaxStorage, quota.used);

        self.broadcast_storage_quota_changed_event(quota)
            .context("Failed to broadcast storage quota changed event")
            .map_err(FatalError)
    }

    #[tracing::instrument(level = "info", skip_all)]
    fn ws_join(
        &mut self,
        socket: Socket,
        client_parameters: ClientParameters,
    ) -> Result<(), FatalError> {
        self.connect_participant(socket, client_parameters)?;
        self.idle_timeout.stop();
        Ok(())
    }

    fn handle_message(
        &mut self,
        message_envelope: MessageEnvelope<SignalingMessage>,
    ) -> Result<(), FatalError> {
        if self
            .lobby_participants
            .get(&message_envelope.participant_id)
            .is_some_and(|participant| {
                participant
                    .all_connection_ids()
                    .any(|id| id == message_envelope.connection_id)
            })
        {
            self.handle_lobby_message(message_envelope)
        } else {
            self.handle_conference_message(message_envelope)
        }
    }

    #[tracing::instrument(level = "info", skip_all, parent = &span)]
    fn handle_conference_message(
        &mut self,
        MessageEnvelope {
            participant_id,
            connection_id,
            message,
            span,
        }: MessageEnvelope<SignalingMessage>,
    ) -> Result<(), FatalError> {
        match message {
            SignalingMessage::Closed(close_reason) => {
                tracing::trace!("Websocket closed: {close_reason:?}");
                let origin = EventOrigin::Participant(ParticipantOrigin {
                    id: participant_id,
                    connection_id,
                    transaction_id: None,
                });
                self.disconnect_participant(origin, participant_id, connection_id, close_reason)
            }

            SignalingMessage::Command(signaling_command) => {
                self.handle_conference_command(signaling_command, participant_id, connection_id)
            }
        }
    }

    #[tracing::instrument(level = "info", skip_all, parent = &span, fields(participant_id = %participant_id))]
    fn handle_lobby_message(
        &mut self,
        MessageEnvelope {
            participant_id,
            connection_id,
            message,
            span,
        }: MessageEnvelope<SignalingMessage>,
    ) -> Result<(), FatalError> {
        match message {
            SignalingMessage::Closed(close_reason) => {
                tracing::trace!("Websocket closed: {close_reason:?}");
                self.disconnect_lobby_participant(participant_id, connection_id);
                Ok(())
            }
            SignalingMessage::Command(signaling_command) => {
                self.handle_lobby_command(signaling_command, participant_id, connection_id)
            }
        }
    }

    fn handle_conference_command(
        &mut self,
        signaling_command: SignalingCommand,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), FatalError> {
        tracing::trace!("received signaling command from conference: {signaling_command:?}");

        let participant_origin = ParticipantOrigin {
            id: participant_id,
            connection_id,
            transaction_id: signaling_command.transaction_id,
        };

        // We currently do not handle the core namespace here because `EnterRoom` as the only core
        // command is only useful in the waiting room.
        match &signaling_command.namespace {
            m if *m == CORE_MODULE_ID => {
                self.handle_conference_core_command(participant_origin, signaling_command);
                Ok(())
            }
            m if *m == BREAKOUT_MODULE_ID => {
                self.handle_breakout_command(participant_origin, signaling_command);
                Ok(())
            }
            _ => {
                let event = DynEvent::WebsocketMessage {
                    participant_id,
                    connection_id,
                    command: signaling_command.payload,
                };
                let Some(state) = self.participants.connected().get(&participant_id) else {
                    tracing::error!("failed to get participant state");
                    return Ok(());
                };
                self.execute_signaling_module_command(
                    participant_origin,
                    state.room,
                    signaling_command.namespace,
                    event,
                )
            }
        }
    }

    fn handle_lobby_command(
        &mut self,
        signaling_command: SignalingCommand,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), FatalError> {
        tracing::trace!("received signaling command from waiting room: {signaling_command:?}");

        let participant_origin = ParticipantOrigin {
            id: participant_id,
            connection_id,
            transaction_id: signaling_command.transaction_id,
        };

        match &signaling_command.namespace {
            m if *m == CORE_MODULE_ID => {
                self.handle_lobby_core_command(participant_origin, signaling_command);
                Ok(())
            }
            _ => {
                let event = DynEvent::WaitingRoomWebsocketMessage {
                    participant_id,
                    connection_id,
                    command: signaling_command.payload,
                };
                self.execute_signaling_module_command(
                    participant_origin,
                    RoomKind::Main,
                    signaling_command.namespace,
                    event,
                )
            }
        }
    }

    fn execute_signaling_module_command(
        &mut self,
        participant_origin: ParticipantOrigin,
        room: RoomKind,
        namespace: ModuleId,
        event: DynEvent,
    ) -> Result<(), FatalError> {
        let Some(module) = self.modules.get_mut(&namespace) else {
            self.handle_unknown_namespace(
                participant_origin.id,
                participant_origin.connection_id,
                participant_origin.transaction_id,
                namespace.to_string(),
            );
            return Ok(());
        };

        let timestamp = Timestamp::now();
        let mut messages = RefCell::new(Vec::new());
        let origin = participant_origin.into();
        let mut ctx = DynModuleContext::new(
            self.info.room_id,
            room,
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
        );
        if let Err(err) = module.on_event(&mut ctx, event) {
            self.handle_fatal_module_error(namespace, participant_origin.transaction_id, err);
        }

        self.handle_module_messages(messages, room, origin, timestamp)
    }

    fn connect_participant(
        &mut self,
        socket: Socket,
        client_parameters: ClientParameters,
    ) -> Result<(), FatalError> {
        let device_id = self.derive_device_id(&client_parameters.device_secret);
        let participant_id = build_participant_id(&client_parameters.kind, device_id);

        // Enforce room participant limit
        if self.exceeds_room_participant_limit(participant_id) {
            tracing::info!(
                "Rejecting new participant '{participant_id}', room participant limit reached"
            );
            Self::participant_limit_reached(socket)?;
            return Ok(());
        }

        // If we ever run into the issue of an uuid collision, a guest could hijack a user session
        // and vice versa. We'd rather decline the new connection when the participant id is
        // known, but the participant kinds differ.
        if let Some(existing_participant) = self.participants.all_unfiltered.get(&participant_id)
            && mem::discriminant(&existing_participant.kind)
                != mem::discriminant(&client_parameters.kind)
        {
            tracing::error!("ParticipantId collision, dropping new participant ({participant_id})");
            return Ok(());
        }

        let Ok(connection_id) = self
            .message_router
            .add_lobby_connection(participant_id, socket)
        else {
            tracing::debug!("rejecting participant connection: already connected");
            return Ok(());
        };

        if let Err(err) =
            self.join_lobby(connection_id, participant_id, device_id, client_parameters)
        {
            tracing::error!("failed to add participant to waiting room {err:#?}");

            self.disconnect_lobby_participant(participant_id, connection_id);
        }

        Ok(())
    }

    /// Returns true when a participant limit is set and adding the `new_participant_id` would
    /// exceed it.
    fn exceeds_room_participant_limit(&self, new_participant_id: ParticipantId) -> bool {
        let Some(limit) = self
            .info
            .room
            .tariff
            .quota(&QuotaType::RoomParticipantLimit)
        else {
            return false;
        };

        // Participants that are already connected do not count towards the limit
        if self.participants.connected().contains(&new_participant_id) {
            return false;
        }

        let participant_count = self
            .participants
            .connected()
            .iter()
            .count()
            .saturating_add(self.lobby_participants.len())
            .try_into()
            .unwrap_or(u64::MAX);

        participant_count >= limit
    }

    fn join_room(
        &mut self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        device_id: DeviceId,
        client_kind: ClientKind,
        display_name: DisplayName,
        mut role: Role,
    ) -> Result<(), FatalError> {
        match self.participants.all_unfiltered.entry(participant_id) {
            Occupied(mut occupied) => {
                let state = occupied.get_mut();
                // Set join/leave timestamps & display name when this is the first device
                if !state.is_connected() {
                    state.joined_at = Utc::now();
                    state.left_at = None;
                    state.display_name = display_name;
                }
                state.in_waiting_room = false;
                state.connections.insert(connection_id, device_id);
                role = state.role;
            }
            Vacant(vacant) => {
                vacant
                    .insert(ParticipantState::new(
                        client_kind.clone(),
                        display_name,
                        role,
                        Utc::now(),
                        false,
                    ))
                    .connections
                    .insert(connection_id, device_id);
            }
        }

        if let Err(err) =
            self.participant_joined(participant_id, connection_id, client_kind.clone(), role)
        {
            tracing::error!("failed to add participant to conference {err:#?}");

            self.disconnect_participant(
                EventOrigin::Internal,
                participant_id,
                connection_id,
                CloseReason::InternalError,
            )?;
        } else {
            self.metrics
                .record_participant_joined(connection_id, (&client_kind).into());
        }

        Ok(())
    }

    fn disconnect_participant(
        &mut self,
        origin: EventOrigin,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        reason: CloseReason,
    ) -> Result<(), FatalError> {
        let Some(state) = self.participants.all_unfiltered.get_mut(&participant_id) else {
            tracing::error!("Attempted to disconnect participant who does not exist");
            return Ok(());
        };

        // When the connection has been removed, the disconnect has already been
        // handled. This is the case when a participant connection has been closed
        // from the server, e.g. when a participant has been kicked. When the
        // connection handle is closed, this function is then called for a second
        // time.
        if state.connections.remove(&connection_id).is_none() {
            return Ok(());
        }
        // Set the left_at timestamp if this was the last connection
        if !state.is_connected() {
            state.left_at = Some(Utc::now());
        }

        self.message_router
            .conference
            .remove_connection(connection_id);

        self.metrics.record_participant_left(connection_id);

        let room = state.room;
        self.participant_disconnected(origin, participant_id, connection_id, room, reason.into())?;

        // start idle timeout when no one is connected
        if !self
            .participants
            .all_unfiltered
            .values()
            .any(ParticipantState::is_connected)
        {
            self.idle_timeout.restart();
        }

        Ok(())
    }

    fn handle_unknown_namespace(
        &mut self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        transaction_id: Option<u64>,
        namespace: String,
    ) {
        tracing::debug!(
            "Received signaling message with unknown namespace: {}",
            &namespace
        );

        let signaling_error = SignalingError::UnknownNamespace {
            invalid_namespace: namespace,
        };

        self.message_router_for_participant(participant_id)
            .send_error(connection_id, transaction_id, signaling_error);
    }

    /// Generate a [`DeviceId`] from a device secret
    ///
    /// This function hashes the device secret and the salt that is configured for the roomserver.
    /// The first 128 bit of the output hash are then used as the uuid for the [`DeviceId`].
    /// This is repeatable, the same device secret will result in the same [`DeviceId`] until
    /// the salt changes because of a roomserver restart.
    ///
    /// Reusing the salt is fine in this case since the salt is private and the device secret
    /// already has a high entropy. In contrast to a password salt, our salt needs to stay
    /// private.
    fn derive_device_id(&self, device_secret: &DeviceSecret) -> DeviceId {
        let mut hasher = blake3::Hasher::new();
        let salt = self.ctx.settings.conference.signaling_salt.as_bytes();
        hasher.update(salt);
        hasher.update(device_secret.to_string().as_bytes());

        let mut uuid_bytes = [0; 16];

        hasher.finalize_xof().fill(&mut uuid_bytes);

        DeviceId::from(Uuid::from_bytes(uuid_bytes))
    }

    fn message_router_for_participant(
        &mut self,
        participant_id: ParticipantId,
    ) -> &mut ScopedRouter {
        if self.lobby_participants.contains_key(&participant_id) {
            &mut self.message_router.lobby
        } else {
            &mut self.message_router.conference
        }
    }
}

fn build_participant_id(kind: &ClientKind, device_id: DeviceId) -> ParticipantId {
    match kind {
        ClientKind::Registered { profile } | ClientKind::RegisteredCallIn { profile } => {
            participant_id_from_uuid(profile.id)
        }
        ClientKind::Guest
        | ClientKind::Recorder { .. }
        | ClientKind::Transcription { .. }
        | ClientKind::CallIn => participant_id_from_uuid(device_id),
    }
}

/// generates a [`ParticipantId`] from something that can be made into a [`Uuid`].
///
/// This should either be the [`DeviceId`] for guests or a [`UserId`] in case of registered users.
fn participant_id_from_uuid(user_id: impl Into<Uuid>) -> ParticipantId {
    ParticipantId::from(user_id.into())
}

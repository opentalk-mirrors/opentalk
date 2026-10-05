// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    cell::RefCell, collections::HashMap, future::Future, marker::PhantomData, sync::Arc,
    time::Duration,
};

use anyhow::Context as _;
use futures::stream::FuturesUnordered;
use opentalk_roomserver_types::{
    client_parameters::{ClientKind, Role},
    connection_id::ConnectionId,
    error::{self, SignalingError},
    kick_reason::KickReason,
    room_kind::RoomKind,
    shared_raw_json::SharedRawJson,
    signaling::module_error::FatalError,
};
use opentalk_types_common::{rooms::RoomId, time::Timestamp, users::UserId};
use opentalk_types_signaling::ParticipantId;
use serde_json::value::RawValue;
use tokio::{
    select,
    sync::oneshot::{self, Receiver, Sender},
};
use tracing::{Instrument as _, debug_span};

use crate::{
    banned_participant::BannedParticipant,
    event_origin::EventOrigin,
    instruction::Instruction,
    internal_module_message::InterModuleMessage,
    lobby_participant::LobbyParticipant,
    loopback::{LoopbackFuture, LoopbackMessage},
    participant_state::{ParticipantState, Participants},
    room_info::RoomTaskInfo,
    signaling_event::SignalingEvent,
    signaling_module::SignalingModule,
    storage::{
        StorageContext,
        assets::{
            AssetMetaData, ModuleAssetStorage, UploadResult,
            provider::{AssetStorageProvider, AssetStream},
        },
        module_resources::{ModuleResourceStorage, provider::ModuleResourceProvider},
    },
};

#[derive(Debug)]
pub enum ModuleMessage {
    Websocket {
        connection_id: ConnectionId,
        message: SharedRawJson,
    },
    WaitingRoomWebsocket {
        connection_id: ConnectionId,
        message: SharedRawJson,
    },
    InternalCommand(InterModuleMessage),
    Instruction(Instruction),
}

/// Contains the room state and provides an interface to send websocket messages.
pub struct ModuleContext<'ctx, M>
where
    M: SignalingModule,
{
    pub room_id: RoomId,
    pub room: RoomKind,
    pub event_origin: EventOrigin,
    pub room_task_info: &'ctx mut RoomTaskInfo,
    /// The websocket messages that are sent out after the module finished its event handling
    messages: &'ctx mut RefCell<Vec<ModuleMessage>>,
    /// Contains all participants including disconnected ones
    pub participants: &'ctx mut Participants,
    pub lobby_participants: &'ctx mut HashMap<ParticipantId, LobbyParticipant>,
    pub banned_participants: &'ctx mut HashMap<ParticipantId, BannedParticipant>,
    pub timestamp: Timestamp,
    loopback_futures: &'ctx mut FuturesUnordered<LoopbackFuture>,
    assets: Arc<dyn AssetStorageProvider>,
    module_resources: Arc<dyn ModuleResourceProvider>,

    m: PhantomData<fn() -> M>,
}

impl<'ctx, M> ModuleContext<'ctx, M>
where
    M: SignalingModule,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        room_id: RoomId,
        room: RoomKind,
        event_origin: EventOrigin,
        room_task_info: &'ctx mut RoomTaskInfo,
        messages: &'ctx mut RefCell<Vec<ModuleMessage>>,
        participants: &'ctx mut Participants,
        lobby_participants: &'ctx mut HashMap<ParticipantId, LobbyParticipant>,
        banned_participants: &'ctx mut HashMap<ParticipantId, BannedParticipant>,
        timestamp: Timestamp,
        loopback_futures: &'ctx mut FuturesUnordered<LoopbackFuture>,
        assets: Arc<dyn AssetStorageProvider>,
        module_resources: Arc<dyn ModuleResourceProvider>,
    ) -> ModuleContext<'ctx, M> {
        Self {
            room_id,
            room,
            event_origin,
            room_task_info,
            messages,
            participants,
            lobby_participants,
            banned_participants,
            timestamp,
            loopback_futures,
            assets,
            module_resources,
            m: PhantomData,
        }
    }

    pub fn reborrow<M2: SignalingModule>(&mut self) -> ModuleContext<'_, M2> {
        ModuleContext {
            room_id: self.room_id,
            room: self.room,
            event_origin: self.event_origin,
            room_task_info: self.room_task_info,
            messages: self.messages,
            participants: self.participants,
            lobby_participants: self.lobby_participants,
            banned_participants: self.banned_participants,
            timestamp: self.timestamp,
            loopback_futures: self.loopback_futures,
            assets: Arc::clone(&self.assets),
            module_resources: Arc::clone(&self.module_resources),
            m: PhantomData,
        }
    }

    /// Send a websocket message of type [`SignalingModule::Outgoing`] to the given
    /// `participant_ids`
    ///
    /// The message is always scoped to the [`SignalingModule::NAMESPACE`]
    ///
    /// # Errors
    ///
    /// Returns `Err` when the [`SignalingModule::Outgoing`] type failed to be serialized.
    pub fn send_ws_message(
        &self,
        participant_ids: impl IntoIterator<Item = ParticipantId>,
        msg: M::Outgoing,
    ) -> Result<(), FatalError> {
        let event = SignalingEvent {
            namespace: M::NAMESPACE,
            transaction_id: self.event_origin.transaction_id(),
            timestamp: Timestamp::now(),
            payload: msg,
        };
        let shared_json: SharedRawJson = serde_json::value::to_raw_value(&event)
            .context("Failed to serialize internal websocket payload type")
            .map_err(FatalError)?
            .into();

        for participant_id in participant_ids {
            let Some(state) = self.participants.connected().get(&participant_id) else {
                tracing::warn!(
                    "Module '{}' attempted to send a websocket message to unknown participant {participant_id}",
                    M::NAMESPACE
                );
                continue;
            };
            let mut messages = self.messages.borrow_mut();

            for (connection_id, ..) in &state.connections {
                messages.push(ModuleMessage::Websocket {
                    connection_id: *connection_id,
                    message: shared_json.clone(),
                });
            }
        }

        Ok(())
    }

    /// Send a websocket message of type [`SignalingModule::Outgoing`] to the given `connection_ids`
    ///
    /// The message is always scoped to the [`SignalingModule::NAMESPACE`]
    ///
    /// # Errors
    ///
    /// Returns `Err` when the [`SignalingModule::Outgoing`] type failed to be serialized.
    pub fn send_ws_message_to_connections(
        &self,
        connection_ids: impl IntoIterator<Item = ConnectionId>,
        msg: M::Outgoing,
    ) -> Result<(), FatalError> {
        let event = SignalingEvent {
            namespace: M::NAMESPACE,
            transaction_id: self.event_origin.transaction_id(),
            timestamp: Timestamp::now(),
            payload: msg,
        };
        let shared_json: SharedRawJson = serde_json::value::to_raw_value(&event)
            .context("Failed to serialize internal websocket payload type")
            .map_err(FatalError)?
            .into();

        let mut messages = self.messages.borrow_mut();
        for connection_id in connection_ids {
            messages.push(ModuleMessage::Websocket {
                connection_id,
                message: shared_json.clone(),
            });
        }

        Ok(())
    }

    pub fn send_ws_message_to_lobby(
        &self,
        participant_ids: impl IntoIterator<Item = ParticipantId>,
        msg: M::Outgoing,
    ) -> Result<(), FatalError> {
        let event = SignalingEvent {
            namespace: M::NAMESPACE,
            transaction_id: self.event_origin.transaction_id(),
            timestamp: Timestamp::now(),
            payload: msg,
        };
        let shared_json: SharedRawJson = serde_json::value::to_raw_value(&event)
            .context("Failed to serialize internal websocket payload type")
            .map_err(FatalError)?
            .into();

        for participant_id in participant_ids {
            let Some(waiting_participant) = self.lobby_participants.get(&participant_id) else {
                tracing::warn!(
                    "Module '{}' attempted to send a websocket message to unknown participant {participant_id}",
                    M::NAMESPACE
                );
                continue;
            };
            let mut messages = self.messages.borrow_mut();

            for connection_id in waiting_participant.all_connection_ids() {
                messages.push(ModuleMessage::WaitingRoomWebsocket {
                    connection_id,
                    message: shared_json.clone(),
                });
            }
        }

        Ok(())
    }

    /// Send a websocket message of type [`SignalingModule::Outgoing`] to the given `connection_ids`
    /// in the lobby
    ///
    /// The message is always scoped to the [`SignalingModule::NAMESPACE`]
    ///
    /// # Errors
    ///
    /// Returns `Err` when the [`SignalingModule::Outgoing`] type failed to be serialized.
    pub fn send_ws_message_to_lobby_connections(
        &self,
        connection_ids: impl IntoIterator<Item = ConnectionId>,
        msg: M::Outgoing,
    ) -> Result<(), FatalError> {
        let event = SignalingEvent {
            namespace: M::NAMESPACE,
            transaction_id: self.event_origin.transaction_id(),
            timestamp: Timestamp::now(),
            payload: msg,
        };
        let shared_json: SharedRawJson = serde_json::value::to_raw_value(&event)
            .context("Failed to serialize internal websocket payload type")
            .map_err(FatalError)?
            .into();

        let mut messages = self.messages.borrow_mut();
        for connection_id in connection_ids {
            messages.push(ModuleMessage::WaitingRoomWebsocket {
                connection_id,
                message: shared_json.clone(),
            });
        }

        Ok(())
    }

    /// Send a websocket command received from one `source_connection` to all
    /// other connections of the same participant.
    ///
    /// The message is always scoped to the [`SignalingModule::NAMESPACE`]
    ///
    /// # Errors
    ///
    /// Returns [`FatalError`] when the [`SignalingEvent`] type failed to be serialized.
    pub fn send_replica(
        &self,
        sender: ParticipantId,
        source_connection: ConnectionId,
        replication_event: M::Outgoing,
    ) -> Result<(), FatalError> {
        let event = SignalingEvent {
            namespace: M::NAMESPACE,
            transaction_id: self.event_origin.transaction_id(),
            timestamp: Timestamp::now(),
            payload: replication_event,
        };

        let shared_json: SharedRawJson = serde_json::value::to_raw_value(&event)
            .context("Failed to serialize internal websocket payload type")
            .map_err(FatalError)?
            .into();

        let Some(state) = self.participants.connected().get(&sender) else {
            tracing::warn!(
                "Module '{}' attempted to replicate a command to unknown participant {sender}",
                M::NAMESPACE
            );
            return Ok(());
        };
        let mut messages = self.messages.borrow_mut();

        for connection_id in state.connections.keys().copied() {
            if connection_id != source_connection {
                messages.push(ModuleMessage::Websocket {
                    connection_id,
                    message: shared_json.clone(),
                });
            }
        }

        Ok(())
    }

    /// Send a command to another [`SignalingModule`]
    ///
    /// * `command` - The command to be sent. The type is defined by the receiving module.
    /// * `handle_result` - Closure that receives the result of the command.
    pub fn send_internal_command<R>(&mut self, command: R::Internal)
    where
        R: SignalingModule,
    {
        let command = InterModuleMessage {
            sender: M::NAMESPACE,
            receiver: R::NAMESPACE,
            command: Box::new(command),
        };
        self.messages
            .get_mut()
            .push(ModuleMessage::InternalCommand(command));
    }

    /// Kick the specified participants
    pub fn kick_participants(&mut self, participants: Vec<ParticipantId>, reason: KickReason) {
        let command = ModuleMessage::Instruction(Instruction::Kick {
            participants,
            reason,
        });
        self.messages.get_mut().push(command);
    }

    pub fn ban_participant(&mut self, participant: ParticipantId) {
        let command = ModuleMessage::Instruction(Instruction::Ban { participant });
        self.messages.get_mut().push(command);
    }

    pub fn broadcast_moderation_role_change(&mut self, participant: ParticipantId) {
        let command =
            ModuleMessage::Instruction(Instruction::BroadcastModerationRoleChange { participant });
        self.messages.get_mut().push(command);
    }

    /// Move the specified participant to the waiting room
    pub fn move_to_waiting_room(&mut self, participant: ParticipantId) {
        let command = ModuleMessage::Instruction(Instruction::MoveToWaitingRoom { participant });
        self.messages.get_mut().push(command);
    }

    /// Invoke an error message of type [`SignalingError`]
    ///
    /// If the event origin is a signaling connection, the error will be sent to the participant.
    ///
    /// The message is always scoped to the [`error::ERROR_MODULE_ID`]
    pub fn handle_error(&self, error: SignalingError) {
        let participant_origin = match self.event_origin {
            EventOrigin::Participant(participant_origin) => participant_origin,
            EventOrigin::Internal => {
                tracing::error!(
                    "Signaling module '{}' returned an error on an event with internal origin: {error:?} ",
                    M::NAMESPACE
                );
                return;
            }
        };

        let event = SignalingEvent {
            namespace: error::ERROR_MODULE_ID,
            transaction_id: self.event_origin.transaction_id(),
            timestamp: Timestamp::now(),
            payload: error,
        };

        let shared_json: SharedRawJson = match serde_json::value::to_raw_value(&event) {
            Ok(value) => value.into(),
            Err(err) => {
                tracing::error!("Failed to serialize SignalingError type: {err}");
                RawValue::from_string(r#"{"error": "internal"}"#.into())
                    .unwrap()
                    .into()
            }
        };

        let mut messages = self.messages.borrow_mut();
        if self
            .participants
            .connected()
            .get(&participant_origin.id)
            .is_some_and(|state| {
                state
                    .connections
                    .contains_key(&participant_origin.connection_id)
            })
        {
            messages.push(ModuleMessage::Websocket {
                connection_id: participant_origin.connection_id,
                message: shared_json,
            });
        } else if self
            .lobby_participants
            .get(&participant_origin.id)
            .is_some_and(|state| {
                state
                    .all_connection_ids()
                    .any(|id| id == participant_origin.connection_id)
            })
        {
            messages.push(ModuleMessage::WaitingRoomWebsocket {
                connection_id: participant_origin.connection_id,
                message: shared_json,
            });
        } else {
            tracing::warn!(
                "Module '{}' attempted to send a websocket error message to unknown participant {} or connection {}",
                M::NAMESPACE,
                participant_origin.id,
                participant_origin.connection_id,
            );
        }
    }

    /// Spawns a new task that completes the given `future` and sends the result
    /// back to the calling module as [`SignalingModule::Loopback`] in the
    /// [`SignalingModule::on_loopback_event`] method.
    ///
    /// The room task will panic if the provided future panics.
    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = M::Loopback> + Send + 'static,
    {
        let origin = self.event_origin;
        let room = self.room;
        let timestamp = self.timestamp;
        let span = debug_span!("spawn");

        let future = future.instrument(span.clone());
        let future = Box::pin(async move {
            Some(LoopbackMessage {
                namespace: M::NAMESPACE,
                origin,
                room,
                timestamp,
                span,
                value: Box::new(future.await),
            })
        });

        self.loopback_futures.push(future);
    }

    /// Spawns a new task that completes the given `future` and sends the result
    /// back to the calling module as [`SignalingModule::Loopback`] in the
    /// [`SignalingModule::on_loopback_event`] method when the result is [`Some`].
    ///
    /// The room task will panic if the provided future panics.
    pub fn spawn_optional<F>(&self, future: F)
    where
        F: Future<Output = Option<M::Loopback>> + Send + 'static,
    {
        let origin = self.event_origin;
        let room = self.room;
        let timestamp = self.timestamp;
        let span = debug_span!("spawn_optional");

        let future = future.instrument(span.clone());
        let future = Box::pin(async move {
            future.await.map(|value| LoopbackMessage {
                namespace: M::NAMESPACE,
                origin,
                room,
                timestamp,
                span,
                value: Box::new(value),
            })
        });

        self.loopback_futures.push(future);
    }

    /// Spawns a blocking function as a asynchronous task and sends the result
    /// back to the calling module as [`SignalingModule::Loopback`] in the
    /// [`SignalingModule::on_loopback_event`] method.
    ///
    /// If the provided function panics, any results will be discarded and the module won't be
    /// notified.
    pub fn spawn_blocking<F>(&self, blocking_function: F)
    where
        F: FnOnce() -> M::Loopback + Send + 'static,
    {
        let span = debug_span!("spawn_blocking");
        let origin = self.event_origin;
        let room = self.room;
        let join_handle = {
            let span = span.clone();
            tokio::task::spawn_blocking(move || span.in_scope(blocking_function))
        };
        let timestamp = self.timestamp;

        let future = Box::pin(async move {
            let Ok(value) = join_handle.await else {
                tracing::error!("module {} panicked in loopback task", M::NAMESPACE);
                return None;
            };

            Some(LoopbackMessage {
                namespace: M::NAMESPACE,
                origin,
                room,
                timestamp,
                span,
                value: Box::new(value),
            })
        });

        self.loopback_futures.push(future);
    }

    /// Creates a loopback future that resolves after the `duration`.
    ///
    /// When `duration` has passed, `create_result` is invoked and the return
    /// value is sent as a loopback event.
    /// Can be cancelled by sending a result into the `rx_cancel` [`Receiver`].
    pub fn loopback_after<F>(&self, duration: Duration, create_result: F) -> Sender<M::Loopback>
    where
        M::Loopback: From<ChannelDroppedError> + Send + Sync + 'static,
        F: FnOnce() -> M::Loopback + Send + Sync + 'static,
    {
        let (tx_cancel, rx_cancel) = oneshot::channel();
        self.spawn(handle_loopback_after(duration, rx_cancel, create_result));
        tx_cancel
    }

    pub fn recv_loopback<F, R>(&self, receiver: oneshot::Receiver<R>, create_result: F)
    where
        M::Loopback: From<ChannelDroppedError> + Send + Sync + 'static,
        F: FnOnce(R) -> M::Loopback + Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        self.spawn(async move {
            match receiver.await {
                Ok(result) => create_result(result),
                Err(_) => ChannelDroppedError.into(),
            }
        });
    }

    pub fn participant_state(&self, participant_id: ParticipantId) -> Option<&ParticipantState> {
        self.participants.all_unfiltered.get(&participant_id)
    }

    pub fn participant_role(&self, participant_id: ParticipantId) -> Option<Role> {
        self.participant_state(participant_id).map(|p| p.role)
    }

    pub fn user_id(&self, participant_id: ParticipantId) -> Option<UserId> {
        self.participant_state(participant_id)
            .and_then(|state| state.kind.user_id())
    }

    pub fn is_moderator(&self, participant_id: ParticipantId) -> bool {
        self.participant_role(participant_id)
            .is_some_and(|r| r.is_moderator())
    }

    pub fn get_client_kind(&self, participant_id: ParticipantId) -> Option<&ClientKind> {
        self.participant_state(participant_id)
            .map(|state| &state.kind)
    }

    pub fn is_room_owner(&self, participant_id: ParticipantId) -> bool {
        let user_id = self
            .participants
            .all_unfiltered
            .get(&participant_id)
            .and_then(|state| state.kind.user_id());
        let Some(user_id) = user_id else {
            return false;
        };
        user_id == self.room_task_info.room.created_by.id
    }

    pub fn assets(&self) -> ModuleAssetStorage {
        ModuleAssetStorage::new(Arc::clone(&self.assets), self.storage_context())
    }

    pub fn module_resources(&self) -> ModuleResourceStorage {
        ModuleResourceStorage::new(Arc::clone(&self.module_resources), self.storage_context())
    }

    fn storage_context(&self) -> StorageContext {
        StorageContext {
            room_id: self.room_id,
            namespace: M::NAMESPACE,
            event: self.room_task_info.room.event.clone(),
        }
    }
}

pub struct ChannelDroppedError;

impl<M> ModuleContext<'_, M>
where
    M: SignalingModule,
    M::Loopback: From<UploadResult>,
{
    pub fn upload_file(&self, asset: AssetStream, metadata: AssetMetaData) {
        let storage_context = self.storage_context();
        let assets = Arc::clone(&self.assets);

        self.spawn(async move {
            assets
                .upload_asset(asset, metadata, &storage_context)
                .await
                .into()
        });
    }
}

async fn handle_loopback_after<F, L>(
    duration: Duration,
    rx_cancel: Receiver<L>,
    create_result: F,
) -> L
where
    F: FnOnce() -> L + 'static,
    L: From<ChannelDroppedError> + Send + Sync + 'static,
{
    select! {
        result = rx_cancel => {
            match result {
                Ok(value) => value,
                Err(_) => ChannelDroppedError.into(),
            }
        },
        () = tokio::time::sleep(duration) => {
            create_result()
        }
    }
}

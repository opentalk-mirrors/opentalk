// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! Manage participant connection tasks

use std::collections::{HashMap, HashSet, hash_map::Entry};

use anyhow::Context;
use futures::{
    SinkExt, Stream,
    stream::{self, SelectAll},
};
pub use message::{CloseReason, MessageEnvelope, SignalingMessage};
use opentalk_roomserver_common::application_state::ApplicationState;
use opentalk_roomserver_signaling::signaling_event::SignalingEvent;
use opentalk_roomserver_types::{
    connection_id::ConnectionId,
    error::{self, SignalingError},
    rate_limit::RateLimitSettings,
    shared_raw_json::SharedRawJson,
    signaling::{
        module_error::FatalError,
        websocket::{CloseFrame, SignalingSocket, SignalingSocketMessage},
    },
};
use opentalk_types_common::{modules::ModuleId, time::Timestamp};
use opentalk_types_signaling::ParticipantId;
use serde::Serialize;
use serde_json::value::RawValue;
use tokio::sync::watch;
use tokio_stream::{StreamExt, wrappers::ReceiverStream};

use crate::{message_router::participant_connection::ConnectionHandle, metrics::Metrics};

mod message;
mod participant_connection;
mod rate_limit;

const WS_CLOSE_ABNORMAL: u16 = 1006;
/// The size of the command channel buffer for each participant connection
const COMMAND_CHANNEL_BUFFER_SIZE: usize = 32;

/// Error that is returned when a new participant is registered with the [`MessageRouter`], but the
/// participant ID already has a connection.
#[derive(Debug, Clone, Copy, thiserror::Error, PartialEq, Eq)]
#[error("The participant already has an active connection")]
pub struct AlreadyConnectedError;

type CommandStream = Box<dyn Stream<Item = MessageEnvelope<SignalingMessage>> + Send + Unpin>;

/// The message router for managing signaling connections
///
/// Provides the interface for communication between client and [`RoomTask`](super::task::RoomTask)
pub struct MessageRouter {
    pub lobby: ScopedRouter,

    pub conference: ScopedRouter,

    /// A stream over the command channels of all active connections
    commands: SelectAll<CommandStream>,
}

impl MessageRouter {
    /// Create a new [`MessageRouter`]
    pub fn new(
        app_state: watch::Receiver<ApplicationState>,
        rate_limit: Option<RateLimitSettings>,
    ) -> Self {
        let mut commands: SelectAll<CommandStream> = SelectAll::new();
        commands.push(Box::new(stream::pending()));

        Self {
            lobby: ScopedRouter::new(app_state.clone(), rate_limit),
            conference: ScopedRouter::new(app_state, rate_limit),
            commands,
        }
    }

    pub fn add_conference_connection<S: SignalingSocket + 'static>(
        &mut self,
        participant_id: ParticipantId,
        websocket: S,
    ) -> Result<ConnectionId, AlreadyConnectedError> {
        self.conference
            .add_connection(participant_id, websocket, &mut self.commands)
    }

    pub fn add_lobby_connection<S: SignalingSocket + 'static>(
        &mut self,
        participant_id: ParticipantId,
        websocket: S,
    ) -> Result<ConnectionId, AlreadyConnectedError> {
        self.lobby
            .add_connection(participant_id, websocket, &mut self.commands)
    }

    /// Upgrade the specified connections from the waiting room to the conference
    pub fn upgrade_connections(&mut self, connections: impl Iterator<Item = ConnectionId>) {
        Self::move_connections(&mut self.lobby, &mut self.conference, connections);
    }

    pub fn move_to_waiting_room(&mut self, connections: impl Iterator<Item = ConnectionId>) {
        Self::move_connections(&mut self.conference, &mut self.lobby, connections);
    }

    fn move_connections(
        from: &mut ScopedRouter,
        to: &mut ScopedRouter,
        connections: impl Iterator<Item = ConnectionId>,
    ) {
        for connection_id in connections {
            if let Some(handle) = from.connections.remove(&connection_id) {
                to.connections.insert(connection_id, handle);
            } else {
                tracing::error!("Trying to move unknown connection {connection_id}");
            }
        }
    }

    /// Receive the next message from any connected participant
    pub async fn recv(&mut self) -> MessageEnvelope<SignalingMessage> {
        // Filter messages from connections that are no longer connected. After
        // the RoomTask initiated a disconnect, commands from the connection remain
        // in the command queue.
        let msg = loop {
            // This should never return `None`, because we push a pending stream into the commands
            let msg = self.commands.next().await.expect("Command stream ended");
            if self.conference.connections.contains_key(&msg.connection_id)
                || self.lobby.connections.contains_key(&msg.connection_id)
            {
                break msg;
            }
        };

        if matches!(msg.message, SignalingMessage::Closed(..)) {
            tracing::debug!(
                "Remove connection {} for participant {}",
                msg.connection_id,
                msg.participant_id,
            );
            self.conference.remove_connection(msg.connection_id);
            self.lobby.remove_connection(msg.connection_id);
        }

        msg
    }

    pub fn disconnected(&mut self) -> HashSet<(ConnectionId, ParticipantId)> {
        self.conference
            .disconnects
            .drain()
            .chain(self.lobby.disconnects.drain())
            .collect()
    }
}

pub struct ScopedRouter {
    /// A collection of active websocket connections
    connections: HashMap<ConnectionId, ConnectionHandle>,

    disconnects: HashMap<ConnectionId, ParticipantId>,

    rate_limit: Option<RateLimitSettings>,

    /// The global application state
    app_state: watch::Receiver<ApplicationState>,
}

impl ScopedRouter {
    /// Create a new [`ScopedRouter`]
    pub fn new(
        app_state: watch::Receiver<ApplicationState>,
        rate_limit: Option<RateLimitSettings>,
    ) -> Self {
        Self {
            connections: HashMap::new(),
            disconnects: HashMap::new(),
            rate_limit,
            app_state,
        }
    }

    fn add_connection<S: SignalingSocket + 'static>(
        &mut self,
        participant_id: ParticipantId,
        mut websocket: S,
        commands: &mut SelectAll<CommandStream>,
    ) -> Result<ConnectionId, AlreadyConnectedError> {
        let connection_id = ConnectionId::generate();

        let entry = self.connections.entry(connection_id);
        let Entry::Vacant(vacant) = entry else {
            tokio::task::spawn(async move {
                websocket
                    .send(SignalingSocketMessage::Close(Some(CloseFrame {
                        code: WS_CLOSE_ABNORMAL,
                        reason: "UUID collision, please retry".into(),
                    })))
                    .await
            });

            return Err(AlreadyConnectedError);
        };
        let (room_task_command_sender, room_task_command_receiver) =
            tokio::sync::mpsc::channel(COMMAND_CHANNEL_BUFFER_SIZE);
        commands.push(Box::new(ReceiverStream::new(room_task_command_receiver)));

        let task_handle = participant_connection::create(
            participant_id,
            connection_id,
            websocket,
            room_task_command_sender,
            self.rate_limit,
            self.app_state.clone(),
        );

        vacant.insert(task_handle);

        Ok(connection_id)
    }

    pub fn remove_connection(&mut self, connection_id: ConnectionId) {
        self.connections.remove(&connection_id);
    }

    /// Send a [`SignalingEvent`] to a participant
    pub fn send_event(
        &mut self,
        participant_connections: impl IntoIterator<Item = ConnectionId>,
        event: SharedRawJson,
    ) {
        for connection_id in participant_connections {
            let Entry::Occupied(handle) = self.connections.entry(connection_id) else {
                if !self.disconnects.contains_key(&connection_id) {
                    tracing::debug!("Tried to sent message to unknown connection");
                }
                continue;
            };

            if let Err(e) = handle.get().send_event(event.clone()) {
                let handle = handle.remove();
                let participant_id = handle.participant_id();

                tracing::debug!(
                    connection_id = %connection_id,
                    participant_id = %participant_id,
                    "Failed to send event, {e}, removing connection",
                );
                self.disconnects
                    .insert(connection_id, handle.participant_id());

                Metrics::increment_congested_connections(1);
            }
        }
    }

    /// Send a [`SignalingEvent`] to **all** participants
    fn broadcast_event(&mut self, event: SharedRawJson) {
        let mut stale_connections = HashSet::new();

        // send events to all participants and collect stale connections
        for (connection_id, handle) in &mut self.connections {
            let cloned_event = event.clone();

            if let Err(e) = handle.send_event(cloned_event) {
                tracing::debug!(
                    connection_id = %connection_id,
                    participant_id = %handle.participant_id(),
                    "Failed to broadcast event, {e}, removing connection",
                );
                stale_connections.insert(*connection_id);
            }
        }

        Metrics::increment_congested_connections(stale_connections.len() as u64);

        // remove all stale connections
        for connection_id in stale_connections {
            if let Some(handle) = self.connections.remove(&connection_id) {
                self.disconnects
                    .insert(connection_id, handle.participant_id());
            }
        }
    }

    /// Send a websocket message to the given list of connections
    ///
    /// # Errors
    ///
    /// Returns a [`FatalError`] when the content fails to serialize
    pub(crate) fn serialize_and_send(
        &mut self,
        connections: impl IntoIterator<Item = ConnectionId>,
        namespace: ModuleId,
        transaction_id: Option<u64>,
        payload: impl Serialize,
    ) -> Result<(), FatalError> {
        let shared_json = Self::serialize_event(namespace, transaction_id, payload)?;
        self.send_event(connections, shared_json);

        Ok(())
    }

    /// Broadcast a websocket message to all participants
    ///
    /// Returns a [`FatalError`] when the content fails to serialize.
    pub(crate) fn serialize_and_broadcast(
        &mut self,
        namespace: ModuleId,
        transaction_id: Option<u64>,
        payload: impl Serialize,
    ) -> Result<(), FatalError> {
        let shared_json = Self::serialize_event(namespace, transaction_id, payload)?;
        self.broadcast_event(shared_json);

        Ok(())
    }

    fn serialize_event(
        namespace: ModuleId,
        transaction_id: Option<u64>,
        payload: impl Serialize,
    ) -> Result<SharedRawJson, FatalError> {
        let event = SignalingEvent {
            namespace,
            transaction_id,
            timestamp: Timestamp::now(),
            payload,
        };
        let shared_json = serde_json::value::to_raw_value(&event)
            .with_context(|| {
                format!(
                    "Failed to serialize message for namespace '{}'",
                    event.namespace
                )
            })
            .map_err(FatalError)?
            .into();

        Ok(shared_json)
    }

    /// Send a websocket error message of type [`SignalingError`] to the associated connection
    ///
    /// The message is always scoped to the [`error::ERROR_MODULE_ID`]
    pub(crate) fn send_error(
        &mut self,
        connection_id: ConnectionId,
        transaction_id: Option<u64>,
        error: SignalingError,
    ) {
        let event = SignalingEvent {
            namespace: error::ERROR_MODULE_ID,
            transaction_id,
            timestamp: Timestamp::now(),
            payload: error,
        };
        let shared_json = match serde_json::value::to_raw_value(&event) {
            Ok(value) => value.into(),
            Err(err) => {
                tracing::error!("Failed to serialize SignalingError type: {err}");
                RawValue::from_string(r#"{"error": "internal"}"#.into())
                    .unwrap()
                    .into()
            }
        };

        self.send_event([connection_id], shared_json);
    }

    /// Send a websocket error message of type [`SignalingError`] to all participants
    ///
    /// The message is always scoped to the [`error::ERROR_MODULE_ID`]
    pub(crate) fn broadcast_error(&mut self, transaction_id: Option<u64>, error: SignalingError) {
        let event = SignalingEvent {
            namespace: error::ERROR_MODULE_ID,
            transaction_id,
            timestamp: Timestamp::now(),
            payload: error,
        };
        let shared_json = match serde_json::value::to_raw_value(&event) {
            Ok(value) => value.into(),
            Err(err) => {
                tracing::error!("Failed to serialize SignalingError type: {err}");
                RawValue::from_string(r#"{"error": "internal"}"#.into())
                    .unwrap()
                    .into()
            }
        };

        self.broadcast_event(shared_json);
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use opentalk_roomserver_common::application_state::ApplicationState;
    use opentalk_roomserver_signaling::signaling_event::SignalingEvent;
    use opentalk_roomserver_types::signaling::websocket::{
        CloseFrame, SignalingSocketItem, SignalingSocketMessage,
    };
    use opentalk_types_common::{modules::module_id, time::Timestamp};
    use opentalk_types_signaling::ParticipantId;
    use serde_json::{json, value::to_raw_value};
    use tokio::sync::watch;

    use crate::{
        message_router::{
            self, MessageEnvelope, MessageRouter, SignalingMessage, WS_CLOSE_ABNORMAL,
        },
        mocking::participant::create_participant_connection,
    };

    #[tokio::test]
    async fn participant_lifecycle() {
        let (_app_state_send, app_state_recv) = watch::channel(ApplicationState::Running);
        let mut router = MessageRouter::new(app_state_recv, None);
        let (p1_socket, p1) = create_participant_connection();
        let p1_id = ParticipantId::from_u128(1);

        let connection = router.add_conference_connection(p1_id, p1_socket).unwrap();

        p1.sender
            .send(Ok(SignalingSocketItem {
                message: SignalingSocketMessage::Close(Some(CloseFrame {
                    code: WS_CLOSE_ABNORMAL,
                    reason: "this is a test".to_string(),
                })),
                done: None,
            }))
            .await
            .unwrap();

        let received = router.recv().await;
        assert_matches!(received, MessageEnvelope {
                participant_id,
                connection_id,
                message: SignalingMessage::Closed(
                    message_router::message::CloseReason::ParticipantClosed
                ),
                ..
            } if participant_id == p1_id && connection_id == connection
        );

        let event = SignalingEvent {
            namespace: module_id!("echo"),
            transaction_id: None,
            timestamp: Timestamp::now(),
            payload: to_raw_value(&json!({
                "cool": 12,
                "thing": true,
            }))
            .unwrap(),
        };
        let shared_json = serde_json::value::to_raw_value(&event).unwrap().into();

        router.conference.send_event([connection], shared_json);
    }
}

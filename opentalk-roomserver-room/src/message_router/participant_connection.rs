// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
//! Manages the websocket connection for a single participant
use std::{pin::Pin, time::Duration};

use bytes::Bytes;
use futures::{
    FutureExt, SinkExt as _, StreamExt as _,
    stream::{self, Peekable},
};
use opentalk_roomserver_common::application_state::ApplicationState;
use opentalk_roomserver_signaling::signaling_event::SignalingEvent;
use opentalk_roomserver_types::{
    connection_id::ConnectionId,
    error::SignalingError,
    rate_limit::RateLimitSettings,
    shared_raw_json::SharedRawJson,
    signaling::{
        SignalingCommand,
        websocket::{
            CloseFrame, SignalingSink, SignalingSocket, SignalingSocketItem,
            SignalingSocketMessage, SignalingStream,
        },
    },
};
use opentalk_types_common::{rooms::RoomId, time::Timestamp};
use opentalk_types_signaling::ParticipantId;
use tokio::sync::{
    mpsc::{self, OwnedPermit},
    watch,
};

use super::{
    message::{CloseReason, MessageEnvelope, SignalingMessage},
    rate_limit::RateLimit,
};

/// The duration the participant has to respond with a close frame after a close
/// frame is sent by the server. If the participant does not send a close frame
/// within this period, the connection will be forcefully terminated.
const CLOSE_TIMEOUT: Duration = Duration::from_secs(5);

/// The interval at which keepalive ping frames are sent to the participant.
///
/// On an otherwise idle signaling connection (e.g. while a SIP/dial-in call is
/// running and media flows over a different channel) this ensures that traffic
/// is sent periodically. Without it, reverse proxies in front of the room server
/// can reach their read/send timeout and reset the connection.
const PING_INTERVAL: Duration = Duration::from_secs(20);

/// The buffer size for events sent to the participant.
const EVENT_BUFFER_SIZE: usize = 256;

/// Handle to the task that communicates with the participant ([`ParticipantConnectionTask`]).
///
/// Dropping this handle will close the connection to the participant.
#[derive(Debug, Clone)]
pub(crate) struct ConnectionHandle {
    /// The [`ParticipantId`] to which this connection belongs to.
    participant_id: ParticipantId,
    /// Event channel to the [`ParticipantConnectionTask`].
    connection_task_event_sender: mpsc::Sender<SharedRawJson>,
}

impl ConnectionHandle {
    /// Instruct the connection task to send an event to the participant.
    ///
    /// If the connection to the participant is already closed or broken, the
    /// event will be dropped and the connection to the participant will be closed.
    ///
    /// ## Timeout
    ///
    /// Sending an event will fail after 1 second. If the participant is
    /// congested, the event is dropped.
    pub fn send_event(
        &self,
        event: SharedRawJson,
    ) -> Result<(), mpsc::error::TrySendError<SharedRawJson>> {
        self.connection_task_event_sender
            .try_send(event)
            .inspect_err(|e| {
                tracing::debug!("Failed to send event to ConnectionTask, {e}");
            })
    }

    pub(crate) fn participant_id(&self) -> ParticipantId {
        self.participant_id
    }
}

/// The reason for exiting the [`ParticipantConnectionTask`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExitReason {
    /// The room task closed the connection
    ClosedByRoomTask,
    /// The client closed the connection
    ClosedByClient,
    /// The websocket was disconnected for no specific reason
    UnexpectedDisconnection,
    /// The room server is shutting down
    Shutdown,
}

impl ExitReason {
    /// Create a close frame appropriate for the exit reason
    fn close_frame(self) -> Option<CloseFrame> {
        match self {
            ExitReason::ClosedByClient | ExitReason::UnexpectedDisconnection => None,
            ExitReason::ClosedByRoomTask => Some(CloseFrame {
                code: 1000,
                reason: "closed by server".into(),
            }),
            ExitReason::Shutdown => Some(CloseFrame {
                code: 1001,
                reason: "server shutdown".into(),
            }),
        }
    }
}

impl From<ExitReason> for CloseReason {
    fn from(value: ExitReason) -> Self {
        match value {
            ExitReason::UnexpectedDisconnection => Self::ConnectionLost,
            ExitReason::Shutdown | ExitReason::ClosedByRoomTask => Self::TaskClosed,
            ExitReason::ClosedByClient => Self::ParticipantClosed,
        }
    }
}

/// Create a new [`ParticipantConnectionTask`]
pub(super) fn create<Socket: SignalingSocket + 'static>(
    room_id: RoomId,
    participant_id: ParticipantId,
    connection_id: ConnectionId,
    socket: Socket,
    room_task_command_sender: mpsc::Sender<MessageEnvelope<SignalingMessage>>,
    rate_limit_settings: Option<RateLimitSettings>,
    app_state: watch::Receiver<ApplicationState>,
) -> ConnectionHandle {
    tracing::debug!(
        ?participant_id,
        ?connection_id,
        "Creating new participant connection task"
    );
    let (event_sender, event_receiver) = mpsc::channel(EVENT_BUFFER_SIZE);

    let (sink, stream) = socket.split();
    let stream = stream.peekable();
    let rate_limit = rate_limit_settings.map(Into::into);
    let this = ParticipantConnectionTask {
        room_id,
        participant_id,
        connection_id,
        room_task_command_sender,
        stream,
        sink,
        room_task_event_receiver: event_receiver,
        app_state,
        rate_limit,
    };
    tokio::task::spawn(this.run());

    ConnectionHandle {
        participant_id,
        connection_task_event_sender: event_sender,
    }
}

/// Handles the messaging between client and [`RoomTask`](super::super::task::RoomTask)
///
/// Messages that were sent by the client get tagged with the associated participant id and will be
/// forwarded to the room task. Messages from the room task will be forwarded to the client.
pub(super) struct ParticipantConnectionTask<Stream: SignalingStream, Sink: SignalingSink> {
    /// The ID of the room this connection is in. Only used for logging purposes.
    room_id: RoomId,
    /// The participant id that is used to tag messages that are send to the room task
    participant_id: ParticipantId,
    /// The connection id of this specific websocket connection
    connection_id: ConnectionId,
    /// The sender to communicate commands to the room task
    room_task_command_sender: mpsc::Sender<MessageEnvelope<SignalingMessage>>,
    /// The participants websocket connection
    stream: Peekable<Stream>,
    sink: Sink,
    /// The receiver to communicate events from the room task
    room_task_event_receiver: mpsc::Receiver<SharedRawJson>,
    /// The application state watch
    app_state: watch::Receiver<ApplicationState>,
    /// Incoming websocket rate limit
    rate_limit: Option<RateLimit>,
}

impl<Stream: SignalingStream, Sink: SignalingSink> ParticipantConnectionTask<Stream, Sink> {
    async fn run(mut self) {
        let exit_reason = self.message_loop().await;

        tracing::debug!("exited message loop");

        self.perform_close_handshake(exit_reason).await;
    }

    /// Loop over a send/receive select until the command channel is closed.
    async fn message_loop(&mut self) -> ExitReason {
        let mut buffer = Vec::with_capacity(EVENT_BUFFER_SIZE);

        // Periodically send a ping to keep the connection alive on idle
        // signaling connections and prevent reverse proxies from timing out.
        let mut ping_interval = tokio::time::interval(PING_INTERVAL);
        ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        // The first tick completes immediately; skip it so we don't ping right
        // after the connection is established.
        ping_interval.tick().await;

        loop {
            let mut stream = Pin::new(&mut self.stream);

            // Build the receive future. We don't bind this to self since this would
            // not allow us to also have a mutable reference in the `self.room_task_event_receiver`
            // (aka send-future).
            let allocated_receive = Self::allocated_receive(
                self.room_task_command_sender.clone(),
                &mut stream,
                self.rate_limit.as_mut(),
            );

            // branches of this select must NOT block
            tokio::select! {
                allocated_msg = allocated_receive => {
                    let InboundMessage {permit, message, should_slow_down} = match allocated_msg {
                        Ok(allocated_msg) => allocated_msg,
                        Err(exit_reason) => return exit_reason,
                    };

                    if should_slow_down
                        && let Err(exit_reason) = self.send_error(SignalingError::SlowDown, None).await {
                            return exit_reason;
                        }

                    if let Err(exit_reason) = self.handle_websocket_frame(message, permit).await {
                        return exit_reason;
                    }
                },
                count = self.room_task_event_receiver.recv_many(&mut buffer, EVENT_BUFFER_SIZE) => {
                    if count == 0 {
                        if self.app_state.borrow().is_shutting_down() {
                            return ExitReason::Shutdown;
                        }

                        return ExitReason::ClosedByRoomTask;
                    };

                    if let Err(e) = self.send_event_to_websocket(&buffer).await {
                        tracing::debug!("Failed to send websocket message: {e}");

                        return ExitReason::UnexpectedDisconnection;
                    }
                    buffer.clear();
                }
                _ = ping_interval.tick() => {
                    if let Err(e) = self.sink.send(SignalingSocketMessage::Ping(Bytes::new())).await {
                        tracing::debug!("Failed to send keepalive ping: {e}");

                        return ExitReason::UnexpectedDisconnection;
                    }
                }
            }
        }
    }

    /// Receive a message and allocate a slot in the [`RoomTask`]-receive-channel.
    ///
    /// This function ensures that we can actually submit the message to the [`RoomTask`]
    /// and that there is a message to submit.
    ///
    /// ## Cancel Safety
    ///
    /// This function needs to be cancel safe, i.e. when canceled no messages must be dropped.
    /// It is expected that this function will be canceled whenever an event is sent to the
    /// participant.
    ///
    /// [`RoomTask`]: crate::room::task::RoomTask
    async fn allocated_receive(
        sender: mpsc::Sender<MessageEnvelope<SignalingMessage>>,
        stream: &mut Pin<&mut Peekable<Stream>>,
        rate_limit: Option<&mut RateLimit>,
    ) -> Result<InboundMessage, ExitReason> {
        // wait until a message arrives
        let _ = stream.as_mut().peek().await;

        let mut should_slow_down = false;
        if let Some(rate_limit) = rate_limit {
            // apply rate limit before consuming the message to ensure back pressure
            should_slow_down = rate_limit.wait_for_token().await;
        }

        // reserve a place in the RoomTask channel
        let Ok(permit) = sender.clone().reserve_owned().await else {
            tracing::debug!("ParticipantConnection closed by room task (channel dropped)");
            return Err(ExitReason::ClosedByRoomTask);
        };

        // `next`-future must be ready since we already peeked above and received a value
        let Some(frame) = stream.next().now_or_never().flatten() else {
            tracing::debug!("Unexpected websocket disconnect (peeked message disappeared)");
            return Err(ExitReason::UnexpectedDisconnection);
        };
        let Ok(message) = frame else {
            tracing::debug!("Error while receiving msg from participant: {frame:?}");
            return Err(ExitReason::UnexpectedDisconnection);
        };

        Ok(InboundMessage {
            message,
            permit,
            should_slow_down,
        })
    }

    #[tracing::instrument(
        parent = None,
        skip_all,
        fields(
            opentalk.room_id = %self.room_id,
            opentalk.participant_id = %self.participant_id,
            opentalk.connection_id = %self.connection_id,
        )
    )]
    async fn handle_websocket_frame(
        &mut self,
        frame: SignalingSocketItem,
        permit: OwnedPermit<MessageEnvelope<SignalingMessage>>,
    ) -> Result<(), ExitReason> {
        let res = match frame.message {
            SignalingSocketMessage::Text(message) => {
                self.handle_text_frame(message.to_string(), permit).await
            }
            SignalingSocketMessage::Close(close_frame) => {
                tracing::debug!("Connection closed by: {close_frame:?}");

                Err(ExitReason::ClosedByClient)
            }
            _ => {
                // Ping, Pong and Binary are ignored
                tracing::debug!("Ignoring ping, pong or binary websocket message");
                Ok(())
            }
        };

        // Mark the message as done.
        if let Some(channel) = frame.done {
            let _ = channel.send(());
        }
        res
    }

    async fn handle_text_frame(
        &mut self,
        msg: String,
        permit: OwnedPermit<MessageEnvelope<SignalingMessage>>,
    ) -> Result<(), ExitReason> {
        let command = match serde_json::from_str::<SignalingCommand>(&msg) {
            Ok(cmd) => cmd,
            Err(e) => {
                tracing::debug!("Error parsing signaling command: {e}");
                let transaction_id = Self::parse_transaction_id(&msg);
                return self.send_error(e, transaction_id).await;
            }
        };
        let wrapped_cmd = SignalingMessage::Command(command)
            .into_envelope(self.connection_id, self.participant_id);

        permit.send(wrapped_cmd);

        Ok(())
    }

    /// Tries to parse only the transaction id of a serialized message
    ///
    /// Returns `None` when the `transaction_id` is missing or the message could
    /// not be parsed at all.
    fn parse_transaction_id(message: &str) -> Option<u64> {
        // Try to parse only the transaction id from the command
        #[derive(serde::Deserialize)]
        struct TransactionId {
            transaction_id: Option<u64>,
        }
        serde_json::from_str::<TransactionId>(message)
            .ok()
            .and_then(|id| id.transaction_id)
    }

    async fn send_error(
        &mut self,
        error: impl Into<SignalingError>,
        transaction_id: Option<u64>,
    ) -> Result<(), ExitReason> {
        let event: SignalingEvent<SignalingError> = SignalingEvent {
            namespace: opentalk_roomserver_types::error::ERROR_MODULE_ID,
            transaction_id,
            timestamp: Timestamp::now(),
            payload: error.into(),
        };
        let error_message = serde_json::to_string(&event)
            .unwrap_or_else(|_| r#"{"error": "internal"}"#.into())
            .into();

        if let Err(e) = self.sink.send(error_message).await {
            tracing::debug!(
                "Failed to send error to participant `{}`: {e}",
                self.participant_id
            );
            return Err(ExitReason::UnexpectedDisconnection);
        }

        Ok(())
    }

    /// Parse the [`SignalingEvent`](opentalk_roomserver_signaling::signaling_event::SignalingEvent)
    /// and send it over the websocket
    async fn send_event_to_websocket(&mut self, events: &[SharedRawJson]) -> anyhow::Result<()> {
        let mut messages = stream::iter(events).map(Self::serialize_message);

        self.sink.send_all(&mut messages).await?;

        Ok(())
    }

    fn serialize_message(event: &SharedRawJson) -> Result<SignalingSocketMessage, Sink::Error> {
        let message = match serde_json::to_string(event) {
            Ok(message) => message,
            Err(e) => {
                let error_msg =
                    format!("Unable to serialize signaling event to websocket message {e}");

                // This error _should_ never occur. We panic if this is a debug build
                if cfg!(debug_assertions) {
                    panic!("{error_msg}");
                }

                tracing::error!("{error_msg}");

                let err: Box<dyn std::error::Error + Send + Sync> = Box::new(e);
                return Err(err.into());
            }
        };
        Ok(message.into())
    }

    /// Perform the close handshake.
    ///
    /// Either the client or the RoomTask initiated the close. If the client initiated the close,
    /// a close frame was already received and the incoming half of the connection is closed. Only
    /// the outgoing half needs to be closed. This will be done by axum automatically (ensure that
    /// the Sink/Stream is polled).
    ///
    /// # Closed by RoomTask
    ///
    /// precondition: the channel that receives events from the RoomTask
    /// (`room_task_event_receiver`) is closed. No close frames have been sent or received.
    ///
    /// 1. We send a close message to the RoomTask, if the channel is still open. The RoomTask most
    ///    likely already dropped this connection, so this might error. If not yet dropped, this
    ///    will ensure that the connection gets removed.
    /// 2. Send a close frame to the client (done automatically by sending remaining events and
    ///    flushing)
    /// 3. Ignore all incoming messages and wait for close frame from client
    ///
    /// # Closed by client
    ///
    /// precondition: channels to the RoomTask are open, stream/ws receiver is closed.
    ///
    /// 1. We send a close message to the RoomTask to ensure this connection gets removed from the
    ///    room.
    /// 2. Send any remaining events until the connection is closed by axum or the RoomTask closes
    ///    the `room_task_event_receiver` channel.
    /// 3. Flush the Sink to ensure the close frame is send (in case no messages were send by the
    ///    RoomTask)
    #[tracing::instrument(skip(self), level = "debug")]
    async fn perform_close_handshake(mut self, exit_reason: ExitReason) {
        let participant_id = self.participant_id;
        let room_task_command_sender = self.room_task_command_sender.clone();

        // In case the channel is full, we don't want to wait until we can process this message.
        drop(tokio::spawn(async move {
            // we don't care if the room tasks command receiver was dropped. There is nothing we can
            // do.
            let _ = room_task_command_sender
                .send(
                    SignalingMessage::Closed(CloseReason::from(exit_reason))
                        .into_envelope(self.connection_id, participant_id),
                )
                .await;
        }));

        // ensure no new events are enqueued
        self.room_task_event_receiver.close();
        // wait until the room task closes this connection and sends remaining messages
        let mut buffer = Vec::with_capacity(EVENT_BUFFER_SIZE);
        while self
            .room_task_event_receiver
            .recv_many(&mut buffer, EVENT_BUFFER_SIZE)
            .await
            > 0
        {
            let mut messages = stream::iter(&buffer).map(Self::serialize_message);

            if let Err(e) = self.sink.send_all(&mut messages).await {
                tracing::debug!("Failed to send websocket messages: {e}");
                break;
            }
            buffer.clear();
        }
        // ensure we at least once call the sink so that the close frame can be send
        if let Err(error) = self.sink.flush().await {
            tracing::debug!("Failed to flush sink {error:?}");
        }

        let Self {
            stream, mut sink, ..
        } = self;

        if let Some(close_frame) = exit_reason.close_frame() {
            tracing::debug!("Send close frame {close_frame:?}");
            let _ = sink
                .send(SignalingSocketMessage::Close(Some(close_frame)))
                .await;
            // Wait for a close frame for the duration of `CLOSE_TIMEOUT` until we forcefully
            // terminate the connection
            if tokio::time::timeout(CLOSE_TIMEOUT, wait_close(stream))
                .await
                .is_err()
            {
                tracing::info!(
                    "Waiting for close frame timed out, client failed to respond in time.",
                );
            }
        }
    }
}

/// Wait for a close frame and discard all other messages
async fn wait_close<Stream: SignalingStream>(mut stream: Stream) {
    while let Some(msg) = stream.next().await {
        match msg {
            Ok(SignalingSocketItem {
                message: SignalingSocketMessage::Close(_),
                ..
            }) => {
                tracing::debug!("Client closed");
                return;
            }
            Err(e) => {
                tracing::debug!("Client dropped connection without close frame: {e:?}");
                return;
            }
            // Discard all messages, but error and close
            Ok(msg) => {
                tracing::debug!("Received message after sending close frame: {msg:?}");
            }
        }
    }
}

struct InboundMessage {
    message: SignalingSocketItem,
    permit: OwnedPermit<MessageEnvelope<SignalingMessage>>,
    should_slow_down: bool,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::{StreamExt as _, pin_mut};
    use opentalk_roomserver_types::{
        connection_id::ConnectionId,
        signaling::{
            SignalingCommand,
            websocket::{SignalingSocketItem, SignalingSocketMessage},
        },
    };
    use opentalk_types_signaling::ParticipantId;
    use tokio::sync::mpsc;
    use tracing::Span;

    use crate::{
        message_router::{
            MessageEnvelope,
            participant_connection::{InboundMessage, ParticipantConnectionTask},
        },
        mocking::{participant::create_participant_connection, socket::MockSocket},
    };

    /// Test that the receive future is cancel safe.
    ///
    /// 1. Fill the command channel for the room task
    /// 2. Send a message from the participant though the socket
    /// 3. call `allocated_receive`, which should block at reserving a queue

    #[tokio::test]
    async fn allocated_receive_cancel_safety() {
        const CMD_CHANNEL_SIZE: usize = 10;
        let (command_tx, mut command_rx) = mpsc::channel(CMD_CHANNEL_SIZE);
        let (p1_socket, p1) = create_participant_connection();
        let p1_socket = p1_socket.peekable();
        pin_mut!(p1_socket);

        while command_tx.capacity() > 0 {
            command_tx
                .send(MessageEnvelope {
                    connection_id: ConnectionId::nil(),
                    participant_id: ParticipantId::from_u128(1),
                    message: crate::message_router::SignalingMessage::Command(
                        SignalingCommand::new(
                            "echo".parse().unwrap(),
                            None,
                            serde_json::value::RawValue::NULL.to_owned(),
                        ),
                    ),
                    span: Span::none(),
                })
                .await
                .unwrap();
        }
        let receive_future = ParticipantConnectionTask::<MockSocket, MockSocket>::allocated_receive(
            command_tx.clone(),
            &mut p1_socket,
            None,
        );

        // Insert a pending message in the socket, that must not get lost when canceling the receive
        // message
        p1.queue_send_ping();

        let timeout = tokio::time::timeout(Duration::from_millis(100), receive_future).await;
        // must be `Elapsed`, since we can never acquired a Reserve for the command channel
        assert!(
            timeout.is_err(),
            "Task must timeout since there is no space in the channel to reserve"
        );

        // receive all messages from the channel. Make space for the one message we queued earlier.
        let mut buf = Vec::with_capacity(CMD_CHANNEL_SIZE);
        command_rx.recv_many(&mut buf, CMD_CHANNEL_SIZE).await;

        // Receive the queued message, which must not have gotten lost.
        let receive_future = tokio::time::timeout(
            Duration::from_millis(100),
            ParticipantConnectionTask::<MockSocket, MockSocket>::allocated_receive(
                command_tx,
                &mut p1_socket,
                None,
            ),
        )
        .await;

        assert!(
            matches!(
                receive_future,
                Ok(Ok(InboundMessage {
                    message: SignalingSocketItem {
                        message: SignalingSocketMessage::Text(..),
                        ..
                    },
                    ..
                })),
            ),
            "Receive expired, but a message should be received"
        )
    }
}

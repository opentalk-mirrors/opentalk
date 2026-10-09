// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{assert_matches, pin::Pin, str::FromStr, time::Duration};

use futures::{
    FutureExt, StreamExt,
    channel::oneshot::{self, Canceled},
    stream::Peekable,
};
use opentalk_roomserver_signaling::{
    signaling_event::SignalingEvent, signaling_module::SignalingModule,
};
use opentalk_roomserver_types::{
    breakout::{
        BREAKOUT_MODULE_ID, breakout_config::BreakoutConfig, command::BreakoutCommand,
        event::BreakoutEvent,
    },
    client_parameters::{ClientKind, ClientParameters},
    connection_id::ConnectionId,
    core::{CORE_MODULE_ID, CoreCommand, CoreEvent},
    join::join_success::JoinSuccess,
    public_user_profile::PublicUserProfile,
    room_kind::RoomKind,
    signaling::{
        SignalingCommand,
        websocket::{self, CloseFrame, SignalingSocketItem, SignalingSocketMessage},
    },
};
use opentalk_types_common::{
    modules::module_id,
    roomserver::DeviceSecret,
    time::TimeZone,
    users::{DisplayName, UserId, UserInfo},
    utils::ExampleData as _,
};
use opentalk_types_signaling::{ParticipantId, Role};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{json, value::to_raw_value};
use tokio::{
    sync::mpsc,
    time::{self, error::Elapsed, timeout},
};
use tokio_stream::wrappers::ReceiverStream;

use super::{
    room::{self, TestRoom},
    socket::MockSocket,
};

const SOCKET_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug, thiserror::Error)]
pub enum ParticipantError {
    #[error("Send")]
    Send(#[from] SendError),
    #[error("Receive")]
    Receive(#[from] ReceiveError),
    #[error("Invalid json")]
    InvalidJson(#[from] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ReceiveError {
    #[error("Closed")]
    Closed,

    #[error("Timeout")]
    Timeout,

    #[error("InvalidJson {message:?}: {error:?}")]
    InvalidJson {
        error: serde_json::Error,
        message: SignalingSocketMessage,
    },

    #[error("UnexpectedMessage {0:?}")]
    UnexpectedMessage(SignalingSocketMessage),

    #[error("UnexpectedCoreEvent {0:?}")]
    UnexpectedCoreEvent(CoreEvent),
}

impl From<Elapsed> for ReceiveError {
    fn from(_value: Elapsed) -> Self {
        Self::Timeout
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error("Closed")]
    Closed,

    #[error("Invalid json: {0:?}")]
    InvalidJson(serde_json::Error),

    #[error("UnexpectedMessage {0:?}")]
    UnexpectedMessage(SignalingSocketMessage),

    #[error("The room task did not acknowledge the message, but dropped the return channel")]
    Canceled,

    #[error("Timeout")]
    Timeout,
}

impl<T> From<mpsc::error::SendError<T>> for SendError {
    fn from(_: mpsc::error::SendError<T>) -> Self {
        SendError::Closed
    }
}

impl From<Elapsed> for SendError {
    fn from(_value: Elapsed) -> Self {
        Self::Timeout
    }
}

pub struct JoinedState {
    join_success: JoinSuccess,
    participant_id: ParticipantId,
    connection_id: ConnectionId,
}

pub struct LobbyState {
    connection_id: ConnectionId,
    participant_id: ParticipantId,
    can_enter: bool,
    display_name: Option<DisplayName>,
}

pub struct WaitingRoomState {
    connection_id: ConnectionId,
    participant_id: ParticipantId,
    display_name: Option<DisplayName>,
}

pub type MockParticipantJoining = MockParticipant<()>;
pub type MockParticipantJoined = MockParticipant<JoinedState>;
pub type MockParticipantLobby = MockParticipant<LobbyState>;
pub type MockParticipantWaiting = MockParticipant<WaitingRoomState>;

pub struct MockParticipant<S> {
    pub(crate) sender: mpsc::Sender<Result<SignalingSocketItem, websocket::Error>>,
    pub(crate) receiver: Peekable<ReceiverStream<SignalingSocketMessage>>,
    pub(crate) state: S,
}

impl<S: std::fmt::Debug> std::fmt::Debug for MockParticipant<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockParticipant")
            .field("sender", &self.sender)
            .field("state", &self.state)
            .finish()
    }
}

impl MockParticipant<()> {
    pub(crate) async fn enter_lobby(mut self) -> Result<MockParticipantLobby, ReceiveError> {
        let event = self.receive().await?;

        if let CoreEvent::JoinedLobby {
            connection_id,
            participant_id,
            device_id: _,
            can_enter,
            display_name,
        } = event.payload
        {
            Ok(MockParticipant {
                sender: self.sender,
                receiver: self.receiver,
                state: LobbyState {
                    connection_id,
                    participant_id,
                    can_enter,
                    display_name,
                },
            })
        } else {
            Err(ReceiveError::UnexpectedCoreEvent(event.payload))
        }
    }
}

impl MockParticipantJoined {
    pub fn join_success(&self) -> &JoinSuccess {
        &self.state.join_success
    }

    pub fn id(&self) -> ParticipantId {
        self.state.participant_id
    }

    pub fn display_name(&self) -> &DisplayName {
        &self.state.join_success.display_name
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.state.connection_id
    }

    pub async fn start_breakout_rooms(
        &mut self,
        others: &mut [&mut MockParticipantJoined],
        config: BreakoutConfig,
    ) -> BreakoutEvent {
        self.send_breakout_command(BreakoutCommand::Start(config), None)
            .await
            .unwrap();

        for p in others {
            assert_matches!(
                p.receive::<BreakoutEvent>().await.unwrap().payload,
                BreakoutEvent::Started { .. },
            );
        }

        self.receive::<BreakoutEvent>().await.unwrap().payload
    }

    pub async fn switch_breakout_room(
        &mut self,
        others: &mut [&mut MockParticipantJoined],
        room: RoomKind,
    ) -> BreakoutEvent {
        self.send_breakout_command(BreakoutCommand::SwitchRoom(room), None)
            .await
            .unwrap();

        for p in others {
            let event = p.receive::<BreakoutEvent>().await.unwrap().payload;
            assert!(
                matches!(event, BreakoutEvent::ParticipantSwitchedRoom { .. },),
                "Error for {}, Expected BreakoutEvent::ParticipantSwitchedRoom, got: {:?}",
                p.id(),
                event
            );
        }

        let event = self.receive::<BreakoutEvent>().await.unwrap().payload;
        assert_matches!(event, BreakoutEvent::SwitchedRoom { .. });
        event
    }

    pub async fn stop_breakout_rooms(
        &mut self,
        others: &mut [&mut MockParticipantJoined],
    ) -> BreakoutEvent {
        self.send_breakout_command(BreakoutCommand::Stop { delay: None }, None)
            .await
            .unwrap();

        for p in others.iter_mut() {
            let event = p.receive::<BreakoutEvent>().await.unwrap().payload;
            assert!(
                matches!(event, BreakoutEvent::Closing { .. }),
                "Expected Closing notice, got: {event:?}"
            );
        }

        for p in others.iter_mut() {
            let event = p.receive::<BreakoutEvent>().await.unwrap().payload;
            assert!(
                matches!(event, BreakoutEvent::Closed),
                "Expected Closed notice, got: {event:?}"
            );
        }

        self.receive::<BreakoutEvent>().await.unwrap().payload
    }

    pub fn in_waiting_room(self) -> MockParticipantWaiting {
        MockParticipantWaiting {
            sender: self.sender,
            receiver: self.receiver,
            state: WaitingRoomState {
                connection_id: self.state.connection_id,
                participant_id: self.state.participant_id,
                display_name: None,
            },
        }
    }
}

impl MockParticipantLobby {
    pub fn can_enter(&self) -> bool {
        self.state.can_enter
    }

    pub fn display_name(&self) -> Option<&DisplayName> {
        self.state.display_name.as_ref()
    }

    pub fn id(&self) -> ParticipantId {
        self.state.participant_id
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.state.connection_id
    }

    /// 1. Send [`CoreCommand::EnterWaitingRoom`]
    /// 2. Receive [`CoreEvent::InWaitingRoom`]
    pub(crate) async fn enter_waiting_room(
        mut self,
        display_name: Option<DisplayName>,
    ) -> Result<MockParticipantWaiting, ParticipantError> {
        self.send_core_command(
            CoreCommand::EnterWaitingRoom {
                display_name: if self.state.display_name.is_some() {
                    None
                } else {
                    display_name
                },
            },
            None,
        )
        .await?;
        let event = self.receive().await?;
        if matches!(event.payload, CoreEvent::InWaitingRoom) {
            Ok(MockParticipantWaiting {
                sender: self.sender,
                receiver: self.receiver,
                state: WaitingRoomState {
                    connection_id: self.state.connection_id,
                    participant_id: self.state.participant_id,
                    display_name: self.state.display_name,
                },
            })
        } else {
            Err(ReceiveError::UnexpectedCoreEvent(event.payload).into())
        }
    }

    /// 1. Send [`CoreCommand::EnterRoom`]
    /// 2. Receive [`CoreEvent::JoinSuccess`]
    pub async fn enter_room(
        mut self,
        display_name: Option<DisplayName>,
    ) -> Result<MockParticipantJoined, ParticipantError> {
        self.send_core_command(
            CoreCommand::EnterRoom {
                display_name: if self.state.display_name.is_some() {
                    None
                } else {
                    display_name
                },
            },
            None,
        )
        .await?;

        let event = self.receive().await?;
        if let CoreEvent::JoinSuccess(msg) = event.payload {
            Ok(MockParticipantJoined {
                sender: self.sender,
                receiver: self.receiver,
                state: JoinedState {
                    join_success: *msg,
                    participant_id: self.state.participant_id,
                    connection_id: self.state.connection_id,
                },
            })
        } else {
            Err(ReceiveError::UnexpectedCoreEvent(event.payload).into())
        }
    }
}

impl MockParticipantWaiting {
    pub fn display_name(&self) -> Option<&DisplayName> {
        self.state.display_name.as_ref()
    }

    /// 1. Receive `ModerationEvent::Accepted`
    /// 2. Send [`CoreCommand::EnterRoom`]
    /// 3. Receive [`JoinSuccess`]
    pub async fn enter_room(
        mut self,
        display_name: Option<DisplayName>,
    ) -> Result<MockParticipantJoined, ParticipantError> {
        let Some(received) = timeout(SOCKET_TIMEOUT, self.receiver.next())
            .await
            .map_err(|err| ParticipantError::Receive(err.into()))?
        else {
            return Err(ReceiveError::Closed.into());
        };
        match received {
            SignalingSocketMessage::Text(text) => {
                let event: SignalingEvent<serde_json::Value> = serde_json::from_str(&text)
                    .map_err(|error| ReceiveError::InvalidJson {
                        error,
                        message: SignalingSocketMessage::Text(text.clone()),
                    })?;

                // The mocking module can not depend on the moderation module, so we
                // can only verify the namespace here.
                if event.namespace != module_id!("moderation") {
                    return Err(
                        ReceiveError::UnexpectedMessage(SignalingSocketMessage::Text(text)).into(),
                    );
                }
            }
            other => return Err(ReceiveError::UnexpectedMessage(other).into()),
        };

        let display_name = if self.state.display_name.is_some() {
            None
        } else {
            display_name
        };
        self.send_core_command(CoreCommand::EnterRoom { display_name }, None)
            .await
            .map_err(ParticipantError::from)?;

        self.join_success().await
    }

    pub async fn join_success(mut self) -> Result<MockParticipantJoined, ParticipantError> {
        let event = self.receive().await?;

        if let CoreEvent::JoinSuccess(msg) = event.payload {
            Ok(MockParticipant {
                sender: self.sender,
                receiver: self.receiver,
                state: JoinedState {
                    join_success: *msg,
                    participant_id: self.state.participant_id,
                    connection_id: self.state.connection_id,
                },
            })
        } else {
            Err(ReceiveError::UnexpectedCoreEvent(event.payload).into())
        }
    }

    pub fn id(&self) -> ParticipantId {
        self.state.participant_id
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.state.connection_id
    }
}

impl<S> MockParticipant<S> {
    pub fn alice(device_number: usize) -> MockParticipantBuilder<PublicUserProfile> {
        let profile = alice_public_profile();

        MockParticipantBuilder {
            profile,
            role: Role::Moderator,
            secret: DeviceSecret::from_str(&format!("Alice Device Secret {device_number}"))
                .expect("Valid device secret"),
        }
    }

    pub fn bob(device_number: usize) -> MockParticipantBuilder<PublicUserProfile> {
        let profile = bob_public_user_profile();

        MockParticipantBuilder {
            profile,
            role: Role::User,
            secret: DeviceSecret::from_str(&format!("Bob Device Secret {device_number}"))
                .expect("Valid device secret"),
        }
    }

    pub fn charlie(device_number: usize) -> MockParticipantBuilder<PublicUserProfile> {
        let profile = charlie_public_user_profile();

        MockParticipantBuilder {
            profile,
            role: Role::User,
            secret: DeviceSecret::from_str(&format!("Charlie Device Secret {device_number}"))
                .expect("Valid device secret"),
        }
    }

    pub fn dave(device_number: usize) -> MockParticipantBuilder<PublicUserProfile> {
        let profile = PublicUserProfile {
            id: UserId::from_u128(0xdae),
            email: "dave@example.com".to_string(),
            user_info: UserInfo {
                title: "".parse().expect("Valid title"),
                firstname: "Dave".to_string(),
                lastname: "Dorsch".to_string(),
                display_name: "Dave the daring".parse().expect("Valid DisplayName"),
                avatar_url: "https://example.com/avatar-of-dave".to_string(),
            },
            timezone: TimeZone::example_data(),
        };

        MockParticipantBuilder {
            profile,
            role: Role::User,
            secret: DeviceSecret::from_str(&format!("Dave Device Secret {device_number}"))
                .expect("Valid device secret"),
        }
    }

    pub fn gustav() -> MockParticipantBuilder<DisplayName> {
        MockParticipantBuilder {
            profile: "Gustav the great".parse().expect("Valid DisplayName"),
            role: Role::User,
            secret: DeviceSecret::from_str("Gustav Device Secret A").expect("Valid device secret"),
        }
    }

    pub fn richard_call_in(device_number: usize) -> MockParticipantBuilder<CallIn> {
        MockParticipantBuilder {
            profile: CallIn("+49 30 12345678".parse().expect("Valid DisplayName")),
            role: Role::User,
            secret: DeviceSecret::from_str(&format!("Richard Device Secret {device_number}"))
                .expect("Valid device secret"),
        }
    }

    pub fn recorder(device_number: usize) -> MockRecorderBuilder {
        MockRecorderBuilder {
            profile: Recorder,
            role: Role::User,
            secret: DeviceSecret::from_str(&format!("Recorder Device Secret {device_number}"))
                .expect("Valid device secret"),
        }
    }

    pub fn transcription(device_number: usize) -> MockTranscriptionBuilder {
        MockTranscriptionBuilder {
            profile: Transcription,
            role: Role::User,
            secret: DeviceSecret::from_str(&format!("Transcription Device Secret {device_number}"))
                .expect("Valid device secret"),
        }
    }

    async fn static_send_ping(
        sender: &mut mpsc::Sender<Result<SignalingSocketItem, websocket::Error>>,
    ) -> Result<(), SendError> {
        sender
            .send(Ok(SignalingSocketItem {
                message: SignalingSocketMessage::Text(
                    json!( {
                        "namespace": "echo",
                        "content": serde_json::Value::Null,
                    })
                    .to_string(),
                ),
                done: None,
            }))
            .await?;
        Ok(())
    }

    pub fn frank(device_number: usize) -> MockParticipantBuilder<PublicUserProfile> {
        let profile = frank_public_user_profile();

        MockParticipantBuilder {
            profile,
            role: Role::Moderator,
            secret: DeviceSecret::from_str(&format!("Frank Device Secret {device_number}"))
                .expect("Valid device secret"),
        }
    }

    pub fn queue_send_ping(&self) {
        let mut sender = self.sender.clone();
        tokio::spawn(async move {
            Self::static_send_ping(&mut sender)
                .await
                .expect("Send should succeed eventually");
        });
    }

    pub async fn send_command<M>(
        &self,
        command: M::Incoming,
        transaction_id: Option<u64>,
    ) -> Result<(), SendError>
    where
        M: SignalingModule,
        M::Incoming: Serialize,
    {
        let command = SignalingCommand {
            namespace: M::NAMESPACE,
            transaction_id,
            payload: to_raw_value(&command).expect("Command must be Serializable"),
        };
        let value = serde_json::to_value(&command).expect("SignalingCommand is serializable");
        self.send_command_raw(value).await
    }

    pub async fn send_command_raw(
        &self,
        command: serde_json::value::Value,
    ) -> Result<(), SendError> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(Ok(SignalingSocketItem {
                message: SignalingSocketMessage::Text(command.to_string()),
                done: Some(tx),
            }))
            .await?;

        if rx.await.is_err() {
            return Err(SendError::Canceled);
        }
        Ok(())
    }

    pub async fn send_breakout_command(
        &self,
        command: BreakoutCommand,
        transaction_id: Option<u64>,
    ) -> Result<(), SendError> {
        let command = SignalingCommand {
            namespace: BREAKOUT_MODULE_ID,
            transaction_id,
            payload: to_raw_value(&command).expect("BreakoutCommand must be serializable"),
        };
        let value = serde_json::to_value(&command).expect("Command is serializable");
        self.send_command_raw(value).await
    }

    pub async fn send_core_command(
        &self,
        command: CoreCommand,
        transaction_id: Option<u64>,
    ) -> Result<(), SendError> {
        let command = SignalingCommand {
            namespace: CORE_MODULE_ID,
            transaction_id,
            payload: to_raw_value(&command).expect("CoreCommand must be serializable"),
        };
        let value = serde_json::to_value(&command).expect("Command is serializable");
        self.send_command_raw(value).await
    }

    pub async fn receive_event<M>(&mut self) -> Result<SignalingEvent<M::Outgoing>, ReceiveError>
    where
        M: SignalingModule,
        M::Outgoing: DeserializeOwned,
    {
        self.receive_event_with_timeout::<M>(SOCKET_TIMEOUT).await
    }

    pub async fn receive_event_with_timeout<M>(
        &mut self,
        timeout: Duration,
    ) -> Result<SignalingEvent<M::Outgoing>, ReceiveError>
    where
        M: SignalingModule,
        M::Outgoing: DeserializeOwned,
    {
        self.receive_timeout(timeout).await
    }

    pub async fn receive<E: DeserializeOwned>(
        &mut self,
    ) -> Result<SignalingEvent<E>, ReceiveError> {
        self.receive_timeout(SOCKET_TIMEOUT).await
    }

    async fn receive_timeout<E: DeserializeOwned>(
        &mut self,
        timeout: Duration,
    ) -> Result<SignalingEvent<E>, ReceiveError> {
        loop {
            let Some(received) = time::timeout(timeout, self.receiver.next()).await? else {
                return Err(ReceiveError::Closed);
            };
            match received {
                SignalingSocketMessage::Text(text) => {
                    let event: SignalingEvent<E> =
                        serde_json::from_str(&text).map_err(|error| ReceiveError::InvalidJson {
                            error,
                            message: SignalingSocketMessage::Text(text),
                        })?;

                    break Ok(event);
                }
                SignalingSocketMessage::Pong(_) => continue,
                SignalingSocketMessage::Ping(msg) => {
                    self.sender
                        .send(Ok(SignalingSocketItem {
                            message: SignalingSocketMessage::Pong(msg),
                            done: None,
                        }))
                        .await
                        .ok();
                    continue;
                }
                other => break Err(ReceiveError::UnexpectedMessage(other)),
            }
        }
    }

    #[must_use]
    pub fn received_nothing(&mut self) -> bool {
        loop {
            match Pin::new(&mut self.receiver).peek().now_or_never() {
                Some(Some(SignalingSocketMessage::Ping(_) | SignalingSocketMessage::Pong(_))) => {
                    // Remove the buffered ping/pong message and keep checking.
                    let _ = self.receiver.next().now_or_never();
                }
                Some(Some(_)) => return false,
                _ => return true,
            }
        }
    }

    pub async fn receive_close_frame(&mut self) -> Result<Option<CloseFrame>, ReceiveError> {
        let Some(received) = timeout(SOCKET_TIMEOUT, self.receiver.next()).await? else {
            return Err(ReceiveError::Closed);
        };
        let SignalingSocketMessage::Close(frame) = received else {
            return Err(ReceiveError::UnexpectedMessage(received));
        };

        Ok(frame)
    }

    pub async fn disconnect(mut self) -> Result<(), ParticipantError> {
        let (tx, rx) = oneshot::channel();

        timeout(
            SOCKET_TIMEOUT,
            self.sender.send(Ok(SignalingSocketItem {
                message: SignalingSocketMessage::Close(Some(CloseFrame {
                    code: 1000,
                    reason: "leaving".to_string(),
                })),
                done: Some(tx),
            })),
        )
        .await
        .map_err(SendError::from)?
        .map_err(SendError::from)?;

        match timeout(SOCKET_TIMEOUT, rx).await {
            Ok(Ok(())) => {}
            Ok(Err(Canceled)) => return Err(SendError::Canceled.into()),
            Err(_) => {
                tracing::debug!("Timeout while waiting for acknowledgement");
                return Err(SendError::Timeout.into());
            }
        }
        tracing::debug!("Close frame sent");
        loop {
            match timeout(SOCKET_TIMEOUT, self.receiver.next()).await {
                Ok(None | Some(SignalingSocketMessage::Close(..))) => {
                    // We won't receive a close frame since the close frame is normally sent by the
                    // websocket implementation and not by the
                    // `ParticipantConnectionTask`. The mocking setup is missing the websocket and
                    // therefore nobody will send the close frame for the RoomServer.
                    return Ok(());
                }
                Ok(Some(..)) => {
                    tracing::debug!("Received event after close frame");
                }
                Err(_) => {
                    return Err(ParticipantError::Receive(ReceiveError::Timeout));
                }
            }
        }
    }
}

pub fn alice_public_profile() -> PublicUserProfile {
    PublicUserProfile {
        id: UserId::from_u128(0xa11ce),
        email: "alice@example.com".to_string(),
        user_info: UserInfo {
            title: "M.Sc.".parse().expect("Valid title"),
            firstname: "Alice".to_string(),
            lastname: "Aal".to_string(),
            display_name: "Alice the angry".parse().expect("Valid DisplayName"),
            avatar_url: "https://example.com/avatar-of-alice".to_string(),
        },
        timezone: TimeZone::example_data(),
    }
}

pub fn bob_public_user_profile() -> PublicUserProfile {
    PublicUserProfile {
        id: UserId::from_u128(0xb0b),
        email: "bob@example.com".to_string(),
        user_info: UserInfo {
            title: "".parse().expect("Valid title"),
            firstname: "Bob".to_string(),
            lastname: "Barsch".to_string(),
            display_name: "Bob the bold".parse().expect("Valid DisplayName"),
            avatar_url: "https://example.com/avatar-of-bob".to_string(),
        },
        timezone: TimeZone::example_data(),
    }
}

pub fn charlie_public_user_profile() -> PublicUserProfile {
    PublicUserProfile {
        id: UserId::from_u128(0xcca211e),
        email: "charlie@example.com".to_string(),
        user_info: UserInfo {
            title: "".parse().expect("Valid title"),
            firstname: "Charlie".to_string(),
            lastname: "Clownfisch".to_string(),
            display_name: "Charlie the charming".parse().expect("Valid DisplayName"),
            avatar_url: "https://example.com/avatar-of-alice".to_string(),
        },
        timezone: TimeZone::example_data(),
    }
}

pub fn frank_public_user_profile() -> PublicUserProfile {
    PublicUserProfile {
        id: UserId::from_u128(0xf9a00),
        email: "frank@example.com".to_string(),
        user_info: UserInfo {
            title: "".parse().expect("Valid title"),
            firstname: "Frank".to_string(),
            lastname: "Forelle".to_string(),
            display_name: "Frank the fabulous".parse().expect("Valid DisplayName"),
            avatar_url: "https://example.com/avatar-of-frank".to_string(),
        },
        timezone: TimeZone::example_data(),
    }
}

pub(crate) fn create_participant_connection() -> (MockSocket, MockParticipantJoining) {
    let websocket_in = mpsc::channel(1);
    let websocket_out = mpsc::channel(1);
    (
        MockSocket::new(websocket_in.1, websocket_out.0),
        MockParticipantJoining {
            sender: websocket_in.0,
            receiver: ReceiverStream::new(websocket_out.1).peekable(),
            state: (),
        },
    )
}

pub struct MockParticipantBuilder<P> {
    profile: P,
    role: Role,
    secret: DeviceSecret,
}

impl<P> MockParticipantBuilder<P> {
    pub fn new(profile: P, secret: DeviceSecret, role: Role) -> Self {
        Self {
            profile,
            role,
            secret,
        }
    }

    pub fn secret(mut self, secret: DeviceSecret) -> Self {
        self.secret = secret;
        self
    }

    pub fn moderator(mut self) -> Self {
        self.role = Role::Moderator;
        self
    }

    pub fn user(mut self) -> Self {
        self.role = Role::User;
        self
    }
}

impl MockParticipantBuilder<PublicUserProfile> {
    pub fn id(mut self, id: UserId) -> Self {
        self.profile.id = id;
        self
    }

    pub fn email(mut self, email: String) -> Self {
        self.profile.email = email;
        self
    }

    /// Panics if the title is invalid
    pub fn title(mut self, title: &str) -> Self {
        self.profile.user_info.title = title.parse().unwrap();
        self
    }

    pub fn firstname(mut self, firstname: String) -> Self {
        self.profile.user_info.firstname = firstname;
        self
    }

    pub fn lastname(mut self, lastname: String) -> Self {
        self.profile.user_info.lastname = lastname;
        self
    }

    /// Panics if the display name is invalid
    pub fn display_name(mut self, display_name: String) -> Self {
        self.profile.user_info.display_name = display_name.parse().unwrap();
        self
    }

    pub fn avatar_url(mut self, avatar_url: String) -> Self {
        self.profile.user_info.avatar_url = avatar_url;
        self
    }

    pub async fn join(self, room: &mut TestRoom) -> Result<MockParticipantJoined, room::Error> {
        room.join_participant(
            ClientParameters {
                device_secret: self.secret,
                kind: ClientKind::Registered {
                    profile: self.profile,
                },
                role: self.role,
            },
            None,
        )
        .await
    }

    pub async fn enter_waiting_room(
        self,
        room: &mut TestRoom,
    ) -> Result<MockParticipantWaiting, room::Error> {
        room.enter_waiting_room(
            ClientParameters {
                device_secret: self.secret,
                kind: ClientKind::Registered {
                    profile: self.profile,
                },
                role: self.role,
            },
            None,
        )
        .await
    }

    pub async fn enter_lobby(
        self,
        room: &mut TestRoom,
    ) -> Result<MockParticipantLobby, room::Error> {
        room.enter_lobby(ClientParameters {
            device_secret: self.secret,
            kind: ClientKind::Registered {
                profile: self.profile,
            },
            role: self.role,
        })
        .await
    }
}

// Guest builder
impl MockParticipantBuilder<DisplayName> {
    /// Panics if the display name is invalid
    pub fn display_name(mut self, display_name: String) -> Self {
        self.profile = display_name.parse().unwrap();
        self
    }

    pub async fn join(self, room: &mut TestRoom) -> Result<MockParticipantJoined, room::Error> {
        room.join_participant(
            ClientParameters {
                device_secret: self.secret,
                kind: ClientKind::Unregistered,
                role: self.role,
            },
            Some(self.profile),
        )
        .await
    }

    pub async fn enter_lobby(
        self,
        room: &mut TestRoom,
    ) -> Result<MockParticipantLobby, room::Error> {
        room.enter_lobby(ClientParameters {
            device_secret: self.secret,
            kind: ClientKind::Unregistered,
            role: self.role,
        })
        .await
    }

    pub async fn enter_waiting_room(
        self,
        room: &mut TestRoom,
    ) -> Result<MockParticipantWaiting, room::Error> {
        room.enter_waiting_room(
            ClientParameters {
                device_secret: self.secret,
                kind: ClientKind::Unregistered,
                role: self.role,
            },
            Some(self.profile),
        )
        .await
    }
}

// Call-in builder
pub struct CallIn(DisplayName);

impl MockParticipantBuilder<CallIn> {
    pub async fn join(self, room: &mut TestRoom) -> Result<MockParticipantJoined, room::Error> {
        room.join_participant(
            ClientParameters {
                device_secret: self.secret,
                kind: ClientKind::CallIn,
                role: self.role,
            },
            Some(self.profile.0),
        )
        .await
    }
}

pub struct Recorder;
pub type MockRecorderBuilder = MockParticipantBuilder<Recorder>;

impl MockRecorderBuilder {
    pub async fn join(
        self,
        room: &mut TestRoom,
        room_kind: RoomKind,
    ) -> Result<MockParticipantJoined, room::Error> {
        room.join_participant(
            ClientParameters {
                device_secret: self.secret,
                kind: ClientKind::Recorder { room: room_kind },
                role: Role::User,
            },
            None,
        )
        .await
    }
}

pub struct Transcription;
pub type MockTranscriptionBuilder = MockParticipantBuilder<Transcription>;

impl MockTranscriptionBuilder {
    pub async fn join(
        self,
        room: &mut TestRoom,
        room_kind: RoomKind,
    ) -> Result<MockParticipantJoined, room::Error> {
        room.join_participant(
            ClientParameters {
                device_secret: self.secret,
                kind: ClientKind::Transcription { room: room_kind },
                role: Role::User,
            },
            None,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use crate::mocking::{
        mock_module::{MockCommand, MockModule},
        room::TestRoom,
    };

    #[test_log::test(tokio::test)]
    async fn received_nothing_ok() {
        let mut room = TestRoom::builder().register_module::<MockModule>().spawn();
        let mut alice = room.join_alice_moderator(0).await;

        alice
            .send_command::<MockModule>(MockCommand::Valid, None)
            .await
            .unwrap();

        // alice must have received something
        assert!(!alice.received_nothing());
    }

    #[test_log::test(tokio::test)]
    async fn received_nothing_error() {
        let mut room = TestRoom::builder().register_module::<MockModule>().spawn();
        let mut alice = room.join_alice_moderator(0).await;

        alice
            .send_command::<MockModule>(MockCommand::Invalid, None)
            .await
            .unwrap();

        // alice must have received something
        assert!(!alice.received_nothing());
    }

    #[test_log::test(tokio::test)]
    async fn received_nothing_panic() {
        let mut room = TestRoom::builder().register_module::<MockModule>().spawn();
        let mut alice = room.join_alice_moderator(0).await;

        alice
            .send_command::<MockModule>(MockCommand::Panic, None)
            .await
            .unwrap();

        // alice must have received something
        assert!(!alice.received_nothing());
    }
}

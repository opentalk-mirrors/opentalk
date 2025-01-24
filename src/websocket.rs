// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Minimal implementation of the opentalk-controller websocket API
//!
//! Should be replaced by more complete implementation sometime in the future

use crate::settings::ControllerSettings;
use anyhow::{bail, Context, Result};
use futures::{SinkExt, StreamExt};
use reqwest::header::SEC_WEBSOCKET_PROTOCOL;
use serde::Deserialize;
use tokio::net::TcpStream;
use tt::tungstenite::client::IntoClientRequest;
use tt::tungstenite::protocol::frame::coding::CloseCode;
use tt::tungstenite::protocol::CloseFrame;
use tt::tungstenite::Message;
use tt::tungstenite::Utf8Bytes;
use tt::MaybeTlsStream;
use tt::WebSocketStream;
use types_signaling::AssociatedParticipant;
use types_signaling::ParticipantId;
use types_signaling_control::state::ControlState;

/// Newtype wrapper for the signaling ticket
pub struct Ticket(pub String);

/// Event received via the websocket
pub enum WebsocketEvent {
    Joined(Participant),
    Update(Participant),
    Left(AssociatedParticipant),
    SessionEnded {
        #[allow(dead_code)]
        issued_by: ParticipantId,
    },
    Disconnected,
}

/// Abstraction over the controller websocket API must call `join` before anything else
pub struct Websocket {
    id: Option<ParticipantId>,
    websocket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

pub enum JoinState {
    Joined(JoinResult),
    InWaitingRoom,
}

impl Websocket {
    /// Connect to the websocket api with the given settings and ticket
    pub async fn connect(settings: &ControllerSettings, ticket: Ticket) -> Result<Self> {
        let uri = if settings.insecure {
            log::warn!("using insecure connection");

            format!("ws://{}/signaling", settings.domain)
        } else {
            format!("wss://{}/signaling", settings.domain)
        };

        let mut ws_req = uri.into_client_request()?;
        ws_req.headers_mut().insert(
            SEC_WEBSOCKET_PROTOCOL,
            format!(
                "opentalk-signaling-json-v1.0,k3k-signaling-json-v1.0,ticket#{}",
                ticket.0
            )
            .try_into()?,
        );

        let (websocket, _) = tt::connect_async(ws_req).await?;

        Ok(Self {
            id: None,
            websocket,
        })
    }

    async fn send(&mut self, data: impl std::fmt::Display) -> Result<()> {
        log::trace!("sending \"{}\"", data);

        self.websocket
            .send(Message::Text(Utf8Bytes::from(data.to_string())))
            .await?;

        Ok(())
    }

    /// Gracefully close the websocket connection
    pub async fn close(&mut self) -> Result<()> {
        self.websocket
            .close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: Utf8Bytes::from_static("none"),
            }))
            .await?;

        Ok(())
    }

    /// Join the room with the given `display_name` and returns a list of participants inside the room
    pub async fn join(&mut self, display_name: &str) -> Result<JoinState> {
        self.send(serde_json::json!({
            "namespace": "control",
            "payload": {
                "action": "join",
                "display_name": display_name
            }
        }))
        .await?;

        let response = self
            .receive_payload()
            .await?
            .context("unexpected disconnection")?;

        match response {
            Payload::Control {
                payload: ControlPayload::JoinSuccess(join_success),
            } => {
                self.id = Some(join_success.id);

                Ok(JoinState::Joined(JoinResult {
                    participants: join_success.participants,
                    livekit_url: join_success.livekit.service_url,
                    livekit_token: join_success.livekit.token,
                }))
            }
            Payload::Moderation {
                payload: ModerationPayload::InWaitingRoom,
            } => Ok(JoinState::InWaitingRoom),
            _ => bail!("Unexpected response to Join request"),
        }
    }

    pub async fn wait_until_accepted_into_waiting_room(&mut self) -> Result<JoinResult> {
        loop {
            match self.receive_payload().await? {
                Some(Payload::Moderation {
                    payload: ModerationPayload::Accepted,
                }) => return self.enter_from_waiting_room().await,
                Some(unhandled) => {
                    log::warn!("Unhandled message while in waiting_room, {unhandled:?}")
                }
                None => bail!("Connection closed while waiting inside the waiting room"),
            }
        }
    }

    pub async fn enter_from_waiting_room(&mut self) -> Result<JoinResult> {
        self.send(serde_json::json!({
            "namespace": "control",
            "payload": {
                "action": "enter_room"
            }
        }))
        .await?;

        let response = self
            .receive_payload()
            .await?
            .context("unexpected disconnection")?;

        match response {
            Payload::Control {
                payload: ControlPayload::JoinSuccess(join_success),
            } => {
                self.id = Some(join_success.id);
                Ok(JoinResult {
                    participants: join_success.participants,
                    livekit_url: join_success.livekit.service_url,
                    livekit_token: join_success.livekit.token,
                })
            }
            _ => bail!("Unexpected response when trying to enter from waiting room"),
        }
    }

    /// Send `raise_hand` or `lower_hand`
    pub async fn send_hand_action(&mut self, raised: bool) -> Result<()> {
        let action = if raised { "raise_hand" } else { "lower_hand" };
        self.send(serde_json::json!({
            "namespace": "control",
            "payload": {
                "action": action
            }
        }))
        .await?;

        Ok(())
    }

    async fn receive_raw(&mut self) -> Result<Option<String>> {
        loop {
            match self.websocket.next().await {
                Some(Ok(Message::Ping(data))) => self.websocket.send(Message::Pong(data)).await?,
                Some(Ok(Message::Close(_))) => return Ok(None),
                Some(Ok(msg)) => return Ok(Some(msg.into_text()?.to_string())),
                Some(Err(e)) => return Err(e.into()),
                None => return Ok(None),
            }
        }
    }

    async fn receive_payload(&mut self) -> Result<Option<Payload>> {
        match self.receive_raw().await? {
            Some(msg) => serde_json::from_str(&msg)
                .context("failed to parse payload")
                .map(Some),
            None => Ok(None),
        }
    }

    /// Wait until either a websocket event or error has been received
    pub async fn receive(&mut self) -> Result<WebsocketEvent> {
        loop {
            match self.receive_payload().await {
                Ok(Some(payload)) => {
                    if let Some(event) = self.payload_to_event(payload)? {
                        return Ok(event);
                    }
                }
                Ok(None) => return Ok(WebsocketEvent::Disconnected),
                Err(e) if e.is::<tt::tungstenite::Error>() => return Err(e),
                Err(_) => {}
            }
        }
    }

    fn payload_to_event(&self, payload: Payload) -> Result<Option<WebsocketEvent>> {
        match payload {
            Payload::Control { payload } => match payload {
                ControlPayload::JoinSuccess(_) => bail!("unexpected JoinSuccess"),
                ControlPayload::Update(participant) => {
                    Ok(Some(WebsocketEvent::Update(participant)))
                }
                ControlPayload::Joined(participant) => {
                    Ok(Some(WebsocketEvent::Joined(participant)))
                }
                ControlPayload::Left(participant) => Ok(Some(WebsocketEvent::Left(participant))),
            },
            Payload::Moderation { payload } => match payload {
                ModerationPayload::InWaitingRoom => bail!("unexpected InWaitingRoom"),
                ModerationPayload::Accepted => bail!("unexpected Accepted"),
                ModerationPayload::SessionEnded(SessionEnded { issued_by }) => {
                    Ok(Some(WebsocketEvent::SessionEnded { issued_by }))
                }
            },
        }
    }
}

pub struct JoinResult {
    pub participants: Vec<Participant>,

    pub livekit_url: String,
    pub livekit_token: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "namespace")]
enum Payload {
    #[serde(rename = "control")]
    Control { payload: ControlPayload },
    #[serde(rename = "moderation")]
    Moderation { payload: ModerationPayload },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
enum ControlPayload {
    JoinSuccess(JoinSuccess),
    Update(Participant),
    Joined(Participant),
    Left(AssociatedParticipant),
}

#[derive(Debug, Deserialize)]
struct JoinSuccess {
    id: ParticipantId,
    participants: Vec<Participant>,
    livekit: Livekit,
}

#[derive(Debug, Deserialize)]
struct Livekit {
    service_url: String,
    token: String,
}

#[derive(Debug, Deserialize)]
pub struct Participant {
    pub id: ParticipantId,
    pub control: ControlState,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
enum ModerationPayload {
    InWaitingRoom,
    Accepted,
    #[allow(dead_code)]
    #[serde(rename = "session_ended")]
    SessionEnded(SessionEnded),
}

#[derive(Debug, Deserialize)]
pub struct SessionEnded {
    pub issued_by: ParticipantId,
}

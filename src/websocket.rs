// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Minimal implementation of the opentalk-controller websocket API
//!
//! Should be replaced by more complete implementation sometime in the future

use crate::media::Target;
use crate::settings::ControllerSettings;
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use futures::{SinkExt, StreamExt};
use reqwest::header::SEC_WEBSOCKET_PROTOCOL;
use serde::Deserialize;
use serde::Serialize;
use std::borrow::Cow;
use tokio::net::TcpStream;
use tt::tungstenite::client::IntoClientRequest;
use tt::tungstenite::protocol::frame::coding::CloseCode;
use tt::tungstenite::protocol::CloseFrame;
use tt::tungstenite::Message;
use tt::MaybeTlsStream;
use tt::WebSocketStream;
use uuid::Uuid;

/// Newtype wrapper for the signaling ticket
pub struct Ticket(pub String);

/// Event received via the websocket
pub enum WebsocketEvent {
    SubscribeOffer {
        offer: String,
        uuid: Uuid,
    },
    PublishResponse {
        response: String,
    },
    CandidateReceived {
        candidate: TrickleCandidate,
        target: Target,
    },
    EndOfCandidate {
        target: Target,
    },
    Joined(Participant),
    Update(Participant),
    Left(AssociatedParticipant),
    Receiving(bool),
    RequestMute {
        issuer: Uuid,
        force: bool,
    },
    SessionEnded {
        issued_by: Uuid,
    },
    Disconnected,
}

/// Abstraction over the controller websocket API must call `join` before anything else
pub struct Websocket {
    id: Option<Uuid>,
    websocket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

pub enum JoinState {
    Joined(Vec<Participant>),
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

        self.websocket.send(Message::Text(data.to_string())).await?;

        Ok(())
    }

    /// Gracefully close the websocket connection
    pub async fn close(&mut self) -> Result<()> {
        self.websocket
            .close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: Cow::Borrowed("none"),
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
                Ok(JoinState::Joined(join_success.participants))
            }
            Payload::Moderation {
                payload: ModerationPayload::InWaitingRoom,
            } => Ok(JoinState::InWaitingRoom),
            _ => bail!("Unexpected response to Join request"),
        }
    }

    pub async fn wait_until_accepted_into_waiting_room(&mut self) -> Result<()> {
        loop {
            match self.receive_payload().await? {
                Some(Payload::Moderation {
                    payload: ModerationPayload::Accepted,
                }) => return Ok(()),
                Some(unhandled) => {
                    log::warn!("Unhandled message while in waiting_room, {unhandled:?}")
                }
                None => bail!("Connection closed while waiting inside the waiting room"),
            }
        }
    }

    pub async fn enter_from_waiting_room(&mut self) -> Result<Vec<Participant>> {
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
                Ok(join_success.participants)
            }
            _ => bail!("Unexpected response when trying to enter from waiting room"),
        }
    }

    /// Initiate negotiation of publish-media using the given SDP
    pub async fn publish(&mut self, sdp: &str) -> Result<()> {
        self.send(serde_json::json!({
            "namespace": "media",
            "payload": {
                "action": "publish",
                "target": self.id.context("id must be set")?,
                "media_session_type": "video",
                "sdp": sdp
            }
        }))
        .await?;

        Ok(())
    }

    /// Request SDP offer for given participant
    pub async fn subscribe(&mut self, uuid: Uuid) -> Result<()> {
        self.send(serde_json::json!({
            "namespace": "media",
            "payload": {
                "action": "subscribe",
                "target": uuid,
                "media_session_type": "video",
                "without_video": true
            }
        }))
        .await?;

        Ok(())
    }

    /// Send a SDP answer to a received SDP offer from the participant
    pub async fn subscribe_answer(&mut self, uuid: Uuid, answer: String) -> Result<()> {
        self.send(serde_json::json!({
            "namespace": "media",
            "payload": {
                "action": "sdp_answer",
                "sdp": answer,
                "target": uuid,
                "media_session_type": "video",
            }
        }))
        .await?;

        Ok(())
    }

    /// Send ICE candidate for webrtc negotiation
    ///
    /// If `uuid` is None the publish session is negotiated,
    /// else its the subscribe session of the given id
    pub async fn send_ice_candidate(
        &mut self,
        uuid: Option<Uuid>,
        candidate: String,
        mline_index: u32,
    ) -> Result<()> {
        self.send(serde_json::json!({
            "namespace": "media",
            "payload": {
                "action": "sdp_candidate",
                "candidate": {
                    "candidate": candidate,
                    "sdpMLineIndex": mline_index,
                },
                "target": uuid.or(self.id).context("uuid must be set")?,
                "media_session_type": "video",
            }
        }))
        .await?;

        Ok(())
    }

    /// Send `end_of_candidates` event for given subscribe session
    pub async fn subscribe_end_of_candidates(&mut self, uuid: Uuid) -> Result<()> {
        self.send(serde_json::json!({
            "namespace": "media",
            "payload": {
                "action": "sdp_end_of_candidates",
                "target": uuid,
                "media_session_type": "video",
            }
        }))
        .await?;

        Ok(())
    }

    /// Send `end_of_candidates` for publish session
    pub async fn publish_end_of_candidates(&mut self) -> Result<()> {
        self.send(serde_json::json!({
            "namespace": "media",
            "payload": {
                "action": "sdp_end_of_candidates",
                "target": self.id.context("id must be set")?,
                "media_session_type": "video",
            }
        }))
        .await?;

        Ok(())
    }

    /// Send `publish_complete` for publish session
    pub async fn publish_complete(&mut self) -> Result<()> {
        self.send(serde_json::json!({
            "namespace": "media",
            "payload": {
                "action": "publish_complete",
                "media_session_type": "video",
                "media_session_state": {
                    "audio": false,
                    "video": false
                }
            }
        }))
        .await?;

        Ok(())
    }

    /// Send `update_media_session` to set the audio mute flag
    pub async fn send_audio_mute(&mut self, mute: bool) -> Result<()> {
        self.send(serde_json::json!({
            "namespace": "media",
            "payload": {
                "action": "update_media_session",
                "media_session_type": "video",
                "media_session_state": {
                    "video": false,
                    "audio": !mute
                }
            }
        }))
        .await?;

        Ok(())
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
                Some(Ok(msg)) => return Ok(Some(msg.into_text()?)),
                Some(Err(e)) => return Err(e.into()),
                None => return Ok(None),
            }
        }
    }

    async fn receive_payload(&mut self) -> Result<Option<Payload>> {
        match self.receive_raw().await? {
            Some(msg) => serde_json::from_str(&msg)
                .context("Failed to parse payload")
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
            Payload::Media { payload } => match payload {
                MediaPayload::SdpOffer(sdp) => Ok(Some(WebsocketEvent::SubscribeOffer {
                    offer: sdp.sdp,
                    uuid: sdp.source.source,
                })),
                MediaPayload::SdpAnswer(sdp) => {
                    Ok(Some(WebsocketEvent::PublishResponse { response: sdp.sdp }))
                }
                MediaPayload::SdpCandidate(sdp) => {
                    let Some(self_id) = self.id else {
                        return Ok(None);
                    };
                    let target = if sdp.source.source == self_id {
                        Target::Publish
                    } else {
                        Target::Subscribe(sdp.source.source)
                    };
                    Ok(Some(WebsocketEvent::CandidateReceived {
                        candidate: sdp.candidate,
                        target,
                    }))
                }
                MediaPayload::SdpEndCandidates(source) => {
                    let Some(self_id) = self.id else {
                        return Ok(None);
                    };
                    let target = if source.source == self_id {
                        Target::Publish
                    } else {
                        Target::Subscribe(self_id)
                    };
                    Ok(Some(WebsocketEvent::EndOfCandidate { target }))
                }
                MediaPayload::MediaStatus(MediaStatus {
                    source,
                    kind,
                    receiving,
                }) => {
                    if let Some(id) = self.id {
                        // Only audio is published so only check for that
                        if source == id && kind == "audio" {
                            Ok(Some(WebsocketEvent::Receiving(receiving)))
                        } else {
                            Ok(None)
                        }
                    } else {
                        Ok(None)
                    }
                }
                MediaPayload::RequestMute(RequestMute { issuer, force }) => {
                    Ok(Some(WebsocketEvent::RequestMute { issuer, force }))
                }
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

#[derive(Debug, Deserialize)]
#[serde(tag = "namespace")]
enum Payload {
    #[serde(rename = "control")]
    Control { payload: ControlPayload },
    #[serde(rename = "media")]
    Media { payload: MediaPayload },
    #[serde(rename = "moderation")]
    Moderation { payload: ModerationPayload },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "message")]
enum MediaPayload {
    #[serde(rename = "sdp_offer")]
    SdpOffer(Sdp),
    #[serde(rename = "sdp_answer")]
    SdpAnswer(Sdp),
    /// SDP Candidate, used for ICE negotiation
    #[serde(rename = "sdp_candidate")]
    SdpCandidate(SdpCandidate),
    /// SDP End of Candidate, used for ICE negotiation
    #[serde(rename = "sdp_end_of_candidates")]
    SdpEndCandidates(Source),
    #[serde(rename = "media_status")]
    MediaStatus(MediaStatus),
    #[serde(rename = "request_mute")]
    RequestMute(RequestMute),
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Sdp {
    /// The payload of the sdp message
    pub sdp: String,

    #[serde(flatten)]
    pub source: Source,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SdpCandidate {
    /// The payload of the sdp message
    pub candidate: TrickleCandidate,

    #[serde(flatten)]
    pub source: Source,
}

/// A candidate for ICE/SDP trickle
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrickleCandidate {
    /// The SDP m-line index
    #[serde(rename = "sdpMLineIndex")]
    pub sdp_m_line_index: u32,

    /// The ICE candidate string
    pub candidate: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Source {
    /// The source of this message
    pub source: Uuid,

    /// The type of stream
    pub media_session_type: MediaSessionType,
}

/// The type of media session
#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaSessionType {
    /// A media session of type video
    #[serde(rename = "video")]
    Video,

    /// A media session of type screen
    #[serde(rename = "screen")]
    Screen,
}

#[derive(Debug, Deserialize)]
struct MediaStatus {
    pub source: Uuid,
    pub kind: String,
    pub receiving: bool,
}

#[derive(Debug, Deserialize)]
pub struct RequestMute {
    pub issuer: Uuid,
    pub force: bool,
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
    id: Uuid,
    participants: Vec<Participant>,
}

#[derive(Debug, Deserialize)]
pub struct AssociatedParticipant {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct Participant {
    pub id: Uuid,

    pub control: ControlData,

    #[serde(default)]
    pub media: MediaData,
}

#[derive(Debug, Deserialize)]
pub struct ControlData {
    pub left_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Default, Deserialize)]
pub struct MediaData {
    pub video: Option<MediaSessionState>,
    pub screen: Option<MediaSessionState>,
}

#[derive(Debug, Deserialize)]
pub struct MediaSessionState {
    pub video: bool,
    pub audio: bool,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
enum ModerationPayload {
    InWaitingRoom,
    Accepted,
    #[serde(rename = "session_ended")]
    SessionEnded(SessionEnded),
}

#[derive(Debug, Deserialize)]
pub struct SessionEnded {
    pub issued_by: Uuid,
}

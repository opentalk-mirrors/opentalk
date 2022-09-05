//! Minimal implementation of the k3k-controller websocket API
//!
//! Should be replaced by more complete implementation sometime in the future

use crate::settings::ControllerSettings;
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use futures::{SinkExt, StreamExt};
use reqwest::header::SEC_WEBSOCKET_PROTOCOL;
use serde::Deserialize;
use std::borrow::Cow;
use tokio::net::TcpStream;
use tt::tungstenite::handshake::client::Request;
use tt::tungstenite::protocol::frame::coding::CloseCode;
use tt::tungstenite::protocol::CloseFrame;
use tt::tungstenite::Message;
use tt::MaybeTlsStream;
use tt::WebSocketStream;
use uuid::Uuid;

/// Newtype wrapper for the signaling ticket
pub struct Ticket(pub String);

#[derive(Deserialize)]
struct Namespaced<'n, P> {
    namespace: &'n str,
    payload: P,
}

/// Event received via the websocket
pub enum WebsocketEvent {
    SubscribeOffer { offer: String, uuid: Uuid },
    PublishResponse { response: String },
    Joined(Participant),
    Update(Participant),
    Left(AssociatedParticipant),
    Receiving(bool),
}

/// Abstraction over the controller websocket API must call `join` before anything else
pub struct Websocket {
    id: Option<Uuid>,
    websocket: WebSocketStream<MaybeTlsStream<TcpStream>>,
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

        let ws_req = Request::get(uri)
            .header(
                SEC_WEBSOCKET_PROTOCOL,
                format!("k3k-signaling-json-v1.0,ticket#{}", ticket.0),
            )
            .body(())?;

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
    pub async fn join(&mut self, display_name: &str) -> Result<Vec<Participant>> {
        #[derive(Debug, Deserialize)]
        struct JoinSuccess {
            id: Uuid,
            participants: Vec<Participant>,
        }

        self.send(serde_json::json!({
            "namespace": "control",
            "payload": {
                "action": "join",
                "display_name": display_name
            }
        }))
        .await?;

        let response = self
            .receive_raw()
            .await
            .context("expected join_success response")?;

        let response: Namespaced<'_, JoinSuccess> = serde_json::from_str(response.to_text()?)?;

        assert_eq!(response.namespace, "control");

        self.id = Some(response.payload.id);

        Ok(response.payload.participants)
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
                    "audio": true,
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

    async fn receive_raw(&mut self) -> Result<Message> {
        loop {
            match self.websocket.next().await {
                Some(Ok(Message::Ping(data))) => self.websocket.send(Message::Pong(data)).await?,
                Some(Ok(msg)) => return Ok(msg),
                Some(Err(e)) => return Err(e.into()),
                None => bail!("connection closed"),
            }
        }
    }

    /// Wait until either a websocket event or error has been received
    pub async fn receive(&mut self) -> Result<WebsocketEvent> {
        loop {
            let msg = self.receive_raw().await?;

            if let Some(event) = self.parse_msg(msg)? {
                return Ok(event);
            }
        }
    }

    fn parse_msg(&self, msg: Message) -> Result<Option<WebsocketEvent>> {
        let msg = msg.into_text()?;

        if let Ok(payload) = serde_json::from_str::<Payload>(&msg) {
            match payload {
                Payload::Control { payload } => match payload {
                    ControlPayload::Update(participant) => {
                        Ok(Some(WebsocketEvent::Update(participant)))
                    }
                    ControlPayload::Joined(participant) => {
                        Ok(Some(WebsocketEvent::Joined(participant)))
                    }
                    ControlPayload::Left(participant) => {
                        Ok(Some(WebsocketEvent::Left(participant)))
                    }
                },
                Payload::Media { payload } => match payload {
                    MediaPayload::SdpOffer(sdp) => Ok(Some(WebsocketEvent::SubscribeOffer {
                        offer: sdp.sdp,
                        uuid: sdp.source,
                    })),
                    MediaPayload::SdpAnswer(sdp) => {
                        Ok(Some(WebsocketEvent::PublishResponse { response: sdp.sdp }))
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
                },
            }
        } else {
            Ok(None)
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "namespace")]
enum Payload {
    #[serde(rename = "control")]
    Control { payload: ControlPayload },
    #[serde(rename = "media")]
    Media { payload: MediaPayload },
}

#[derive(Deserialize)]
#[serde(tag = "message")]
enum MediaPayload {
    #[serde(rename = "sdp_offer")]
    SdpOffer(Sdp),
    #[serde(rename = "sdp_answer")]
    SdpAnswer(Sdp),
    #[serde(rename = "media_status")]
    MediaStatus(MediaStatus),
}

#[derive(Deserialize)]
struct Sdp {
    pub sdp: String,
    pub source: Uuid,
}

#[derive(Deserialize)]
struct MediaStatus {
    pub source: Uuid,
    pub kind: String,
    pub receiving: bool,
}

#[derive(Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
enum ControlPayload {
    Update(Participant),
    Joined(Participant),
    Left(AssociatedParticipant),
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

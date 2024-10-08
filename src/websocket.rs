// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Minimal implementation of the opentalk-controller websocket API
//!
//! Should be replaced by more complete implementation sometime in the future

use crate::media::Target;
use crate::settings::ControllerSettings;
use anyhow::{bail, Context, Result};
use compositor::MediaDescriptor;
use futures::{SinkExt, StreamExt};
use reqwest::header::SEC_WEBSOCKET_PROTOCOL;
use serde::Deserialize;
use std::borrow::Cow;
use tokio::net::TcpStream;
use tt::tungstenite::client::IntoClientRequest;
use tt::tungstenite::protocol::frame::coding::CloseCode;
use tt::tungstenite::protocol::CloseFrame;
use tt::tungstenite::Message;
use tt::MaybeTlsStream;
use tt::WebSocketStream;
use types::signaling::control::state::ControlState;
use types::signaling::control::AssociatedParticipant;
use types::signaling::media::event::{MediaEvent, MediaStatus, RequestMute};
use types::signaling::media::{MediaSessionState, MediaSessionType, TrickleCandidate};
use types_signaling::ParticipantId;

/// Newtype wrapper for the signaling ticket
pub struct Ticket(pub String);

/// Event received via the websocket
pub enum WebsocketEvent {
    SubscribeOffer {
        offer: String,
        desc: MediaDescriptor,
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
    Receiving(MediaSessionType, bool),
    RequestMute {
        #[allow(dead_code)]
        issuer: ParticipantId,
        #[allow(dead_code)]
        force: bool,
    },
    SpeakerUpdated {
        speaker: ParticipantId,
        speaking: bool,
    },
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

    pub async fn wait_until_accepted_into_waiting_room(&mut self) -> Result<Vec<Participant>> {
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
    pub async fn publish(&mut self, sdp: &str, media_session_type: MediaSessionType) -> Result<()> {
        self.send(serde_json::json!({
            "namespace": "media",
            "payload": {
                "action": "publish",
                "target": self.id.context("id must be set")?,
                "media_session_type": media_session_type,
                "sdp": sdp
            }
        }))
        .await?;

        Ok(())
    }

    /// Request SDP offer for given participant
    pub async fn subscribe(&mut self, desc: MediaDescriptor, with_video: bool) -> Result<()> {
        self.send(serde_json::json!({
            "namespace": "media",
            "payload": {
                "action": "subscribe",
                "target": desc.participant_id,
                "media_session_type": desc.media_type,
                "without_video": !with_video
            }
        }))
        .await?;

        Ok(())
    }

    /// Send a SDP answer to a received SDP offer from the participant
    pub async fn subscribe_answer(&mut self, desc: MediaDescriptor, answer: String) -> Result<()> {
        self.send(serde_json::json!({
            "namespace": "media",
            "payload": {
                "action": "sdp_answer",
                "sdp": answer,
                "target": desc.participant_id,
                "media_session_type": desc.media_type,
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
        participant_id: Option<ParticipantId>,
        media_session_type: MediaSessionType,
        candidate: String,
        mline_index: u32,
    ) -> Result<()> {
        let target = participant_id.or(self.id).context("self.id must be set")?;

        self.send(serde_json::json!({
            "namespace": "media",
            "payload": {
                "action": "sdp_candidate",
                "candidate": {
                    "candidate": candidate,
                    "sdpMLineIndex": mline_index,
                },
                "target": target,
                "media_session_type": media_session_type,
            }
        }))
        .await?;

        Ok(())
    }

    /// Send `end_of_candidates` event for given subscribe session
    pub async fn subscribe_end_of_candidates(&mut self, desc: MediaDescriptor) -> Result<()> {
        self.send(serde_json::json!({
            "namespace": "media",
            "payload": {
                "action": "sdp_end_of_candidates",
                "target": desc.participant_id,
                "media_session_type": desc.media_type,
            }
        }))
        .await?;

        Ok(())
    }

    /// Send `end_of_candidates` for publish session
    pub async fn publish_end_of_candidates(
        &mut self,
        media_session_type: MediaSessionType,
    ) -> Result<()> {
        self.send(serde_json::json!({
            "namespace": "media",
            "payload": {
                "action": "sdp_end_of_candidates",
                "target": self.id.context("id must be set")?,
                "media_session_type": media_session_type,
            }
        }))
        .await?;

        Ok(())
    }

    /// Send `publish_complete` for publish session
    pub async fn publish_complete(
        &mut self,
        with_video: bool,
        media_session_type: MediaSessionType,
    ) -> Result<()> {
        self.send(serde_json::json!({
            "namespace": "media",
            "payload": {
                "action": "publish_complete",
                "media_session_type": media_session_type,
                "media_session_state": {
                    "audio": true,
                    "video": with_video
                }
            }
        }))
        .await?;

        Ok(())
    }

    /// Send `update_media_session` to set the audio mute flag
    pub async fn send_mute(
        &mut self,
        audio_muted: bool,
        video_muted: bool,
        media_session_type: MediaSessionType,
    ) -> Result<()> {
        self.send(serde_json::json!({
            "namespace": "media",
            "payload": {
                "action": "update_media_session",
                "media_session_type": media_session_type,
                "media_session_state": {
                    "video": !video_muted,
                    "audio": !audio_muted,
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
            Payload::Media { payload } => match payload {
                MediaEvent::SdpOffer(sdp) => Ok(Some(WebsocketEvent::SubscribeOffer {
                    offer: sdp.sdp,
                    desc: sdp.source.into(),
                })),
                MediaEvent::SdpAnswer(sdp) => {
                    Ok(Some(WebsocketEvent::PublishResponse { response: sdp.sdp }))
                }
                MediaEvent::SdpCandidate(sdp) => {
                    let Some(self_id) = self.id else {
                        return Ok(None);
                    };
                    let target = if sdp.source.source == self_id {
                        Target::Publish(sdp.source.media_session_type)
                    } else {
                        Target::Subscribe(sdp.source.into())
                    };
                    Ok(Some(WebsocketEvent::CandidateReceived {
                        candidate: sdp.candidate,
                        target,
                    }))
                }
                MediaEvent::SdpEndOfCandidates(source) => {
                    let Some(self_id) = self.id else {
                        return Ok(None);
                    };
                    let target = if source.source == self_id {
                        Target::Publish(source.media_session_type)
                    } else {
                        Target::Subscribe(source.into())
                    };
                    Ok(Some(WebsocketEvent::EndOfCandidate { target }))
                }
                MediaEvent::MediaStatus(MediaStatus {
                    source,
                    kind: _,
                    receiving,
                }) => {
                    if let Some(id) = self.id {
                        if source.source == id {
                            return Ok(Some(WebsocketEvent::Receiving(
                                source.media_session_type,
                                receiving,
                            )));
                        }
                    }

                    Ok(None)
                }
                MediaEvent::RequestMute(RequestMute { issuer, force }) => {
                    Ok(Some(WebsocketEvent::RequestMute { issuer, force }))
                }
                MediaEvent::SpeakerUpdated(event) => Ok(Some(WebsocketEvent::SpeakerUpdated {
                    speaker: event.participant,
                    speaking: event.speaker.is_speaking,
                })),
                evt => {
                    log::debug!("Unhandled media event: {evt:?}");
                    Ok(None)
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
    Media { payload: MediaEvent },
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
}

#[derive(Debug, Deserialize)]
pub struct Participant {
    pub id: ParticipantId,
    pub control: ControlState,

    #[serde(default)]
    pub media: MediaData,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct MediaData {
    pub video: Option<MediaSessionState>,
    #[allow(dead_code)]
    pub screen: Option<MediaSessionState>,
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

// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

#[cfg(feature = "actix")]
pub mod actix;
#[cfg(feature = "axum")]
pub mod axum;
pub mod websocket;

use http::HeaderMap;
use opentalk_types_common::rooms::RoomId;
use opentalk_types_signaling::ParticipantId;
use tokio_tungstenite::MaybeTlsStream;
use uuid::Uuid;

use crate::{connection_id::ConnectionId, room_kind::RoomKind};

#[derive(Debug, Clone, Copy)]
pub enum LiveKitProxyTarget {
    LiveKit { room_kind: RoomKind },
    SubroomAudio { whisper_id: Uuid },
}

#[derive(Debug, Clone)]
pub struct LiveKitProxyRequest {
    pub raw_query: Option<String>,
    pub headers: HeaderMap,
    pub room_id: RoomId,
    pub proxy_target: LiveKitProxyTarget,
    pub participant_id: ParticipantId,
    pub connection_id: ConnectionId,
}

/// The type of an upstream WebSocket connection to a LiveKit server.
pub type PreparedSocket = tokio_tungstenite::WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
//! Exposed message types
use opentalk_roomserver_types::{
    disconnect_reason::DisconnectReason, kick_reason::KickReason, signaling::SignalingCommand,
};
use opentalk_types_signaling::ParticipantId;

use super::ConnectionId;

/// The reason for the closed participant connection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseReason {
    /// The participant initiated the close by sending a WebSocket close frame.
    ParticipantClosed,

    /// The connection to the participant was closed without a close frame.
    ConnectionLost,

    /// The connection to the participant was closed by sending a close frame to the participant.
    TaskClosed,

    /// The participant was kicked by a moderator
    Kicked,

    /// The participant was removed from the meeting because debriefing was started
    Debriefed,

    /// The participant was banned from the room
    Banned,

    /// The participant is a guest and guest access has been disabled
    GuestAccessDisabled,

    InternalError,
}

impl From<CloseReason> for DisconnectReason {
    fn from(value: CloseReason) -> Self {
        match value {
            CloseReason::ParticipantClosed => DisconnectReason::Leave,
            CloseReason::ConnectionLost => DisconnectReason::ConnectionLost,
            CloseReason::Kicked => DisconnectReason::Kicked,
            CloseReason::Debriefed => DisconnectReason::Debriefed,
            CloseReason::Banned => DisconnectReason::Banned,
            CloseReason::GuestAccessDisabled => DisconnectReason::GuestAccessDisabled,
            CloseReason::InternalError | CloseReason::TaskClosed => DisconnectReason::InternalError,
        }
    }
}

impl From<KickReason> for CloseReason {
    fn from(value: KickReason) -> Self {
        match value {
            KickReason::Kicked => CloseReason::Kicked,
            KickReason::Debriefed => CloseReason::Debriefed,
            KickReason::GuestAccessDisabled => CloseReason::GuestAccessDisabled,
        }
    }
}

/// A signaling message sent by a client
#[derive(Debug)]
pub enum SignalingMessage {
    /// The connection to the client was closed
    Closed(CloseReason),
    /// A signaling command for a specific signaling module.
    Command(SignalingCommand),
}

impl SignalingMessage {
    pub fn into_envelope(
        self,
        connection_id: ConnectionId,
        participant_id: ParticipantId,
    ) -> MessageEnvelope<SignalingMessage> {
        MessageEnvelope {
            participant_id,
            connection_id,
            message: self,
            span: tracing::Span::current(),
        }
    }
}

#[derive(Debug)]
pub struct MessageEnvelope<M> {
    pub participant_id: ParticipantId,
    pub connection_id: ConnectionId,
    pub message: M,
    pub span: tracing::Span,
}

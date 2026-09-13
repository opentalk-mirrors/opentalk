// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_types::connection_id::ConnectionId;
use opentalk_types_signaling::ParticipantId;

/// The origin of a event in the roomserver
///
/// This includes events from signaling sessions, web api requests and internal events.
#[derive(Debug, Clone, Copy)]
pub enum EventOrigin {
    Participant(ParticipantOrigin),

    /// Some form of internal event, e.g. a timer event or internal state change
    Internal,
}

impl EventOrigin {
    pub fn participant_id(&self) -> Option<ParticipantId> {
        match self {
            EventOrigin::Participant(participant_origin) => Some(participant_origin.id),
            _ => None,
        }
    }

    pub fn transaction_id(&self) -> Option<u64> {
        match self {
            EventOrigin::Participant(participant_origin) => participant_origin.transaction_id,
            _ => None,
        }
    }
}

impl From<ParticipantOrigin> for EventOrigin {
    fn from(participant_origin: ParticipantOrigin) -> Self {
        Self::Participant(participant_origin)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ParticipantOrigin {
    pub id: ParticipantId,
    pub connection_id: ConnectionId,
    pub transaction_id: Option<u64>,
}

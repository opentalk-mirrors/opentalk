// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use opentalk_roomserver_types::{
    client_parameters::{ClientKind, Role},
    connection_id::ConnectionId,
    device_id::DeviceId,
};

/// Information associated with a participant that joined the waiting room.
#[derive(Debug, Clone)]
pub struct LobbyParticipant {
    /// The kind of the participant
    pub kind: ClientKind,

    /// The role that the participant assumes in the meeting.
    pub role: Role,

    /// All connections and their associated devices
    pub connections: HashMap<ConnectionId, DeviceId>,

    /// Whether the participant was accepted to enter the meeting
    pub accepted: bool,

    /// The time when the participant joined the waiting room
    pub joined_at: DateTime<Utc>,
}

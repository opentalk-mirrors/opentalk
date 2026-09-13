// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::time::Duration;

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BreakoutConfig {
    /// The breakout rooms and their config, their [`BreakoutId`](super::breakout_id::BreakoutId)
    /// is equal to the index in the list
    pub rooms: Vec<BreakoutRoomConfig>,

    /// The duration for all breakout rooms, minimum of 60 seconds.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "opentalk_types_common::utils::duration_seconds_option"
    )]
    pub duration: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BreakoutRoomConfig {
    /// The name of the breakout room
    pub name: String,

    /// The breakout assignments for participants, these are not strictly enforced by the
    /// roomserver.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assignments: Vec<ParticipantId>,
}

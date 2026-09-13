// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "reason")]
/// Reason that is provided when the automod session ends
pub enum StoppedReason {
    /// The session was stopped by a moderator
    StoppedByModerator {
        /// The moderator who issued the stop command
        issued_by: ParticipantId,
    },

    /// All participants of the automod session yielded
    SessionFinished,
}

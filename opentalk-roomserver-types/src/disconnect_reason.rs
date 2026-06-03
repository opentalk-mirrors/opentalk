// SPDX-License-Identifier: EUPL-1.2
//
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisconnectReason {
    /// The participant left the conference
    Leave,
    /// The connection was interrupted
    ConnectionLost,
    /// The participant was kicked by a moderator
    Kicked,
    /// The participant was removed from the conference because debriefing was started
    Debriefed,
    /// The participant was banned from the room
    Banned,
    /// The participant was sent to the waiting room
    SentToWaitingRoom,
    /// The participant is a guest and guest access has been disabled
    GuestAccessDisabled,
    /// The participant was removed due to an internal error
    InternalError,
}

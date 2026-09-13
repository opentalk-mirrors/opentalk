// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

/// The reason why presence logging started.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceLoggingStartedReason {
    /// Automatically started as configured for the meeting
    Autostart,

    /// The creator started presence logging manually while other participants
    /// were already present in the room.
    StartedManually,
}

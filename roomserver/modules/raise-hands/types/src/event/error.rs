// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types::signaling::module_error::ModuleError;
use serde::{Deserialize, Serialize};

/// Error from the `moderation` module namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum RaiseHandsError {
    InsufficientPermissions,
    /// The requested participant is not connected
    UnknownParticipant,
    /// Attempted to raise hand while handraising is disabled for the meeting
    RaiseHandsDisabled,
}

impl ModuleError for RaiseHandsError {}

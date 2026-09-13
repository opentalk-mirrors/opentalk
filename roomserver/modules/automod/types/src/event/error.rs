// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types::signaling::module_error::ModuleError;
use serde::{Deserialize, Serialize};

/// A command from the frontend has triggered an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "error")]
pub enum AutomodError {
    /// The selection made by the frontend was invalid.
    ///
    /// Can originate from the `start`, `yield` or `select` command.
    InvalidSelection,

    /// The issued command can only be issued by a moderator, but the issuer isn't one.
    InsufficientPermissions,

    /// An automod session is already running.
    SessionAlreadyRunning,

    /// No automod session is running.
    SessionNotRunning,

    /// The edit command is invalid.
    InvalidEdit,

    /// An internal error occurred.
    Internal,
}

impl ModuleError for AutomodError {}

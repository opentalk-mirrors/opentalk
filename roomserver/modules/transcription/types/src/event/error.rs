// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2
use opentalk_roomserver_types::signaling::module_error::ModuleError;
use serde::{Deserialize, Serialize};

/// Error from the `transcription` module namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum TranscriptionError {
    /// The request to start the transcription failed
    ServiceRequestFailed,

    /// The transcription feature is not available in this room
    FeatureDisabled,

    /// The participant has insufficient permissions to perform a command
    InsufficientPermissions,

    /// The transcription is already active in this room
    AlreadyActive,

    /// Tried to pause or stop a transcription, but it isn't active
    NotActive,

    /// The transcription service disconnected unexpectedly
    ServiceDisconnected,
}

impl ModuleError for TranscriptionError {}

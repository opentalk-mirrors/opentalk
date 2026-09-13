// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types::signaling::module_error::ModuleError;
use serde::{Deserialize, Serialize};

/// Error from the `recording` module namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum RecordingError {
    /// The recording feature is not available in this room
    RecordFeatureDisabled,

    /// The streaming feature is not available in this room
    StreamFeatureDisabled,

    /// The participant has insufficient permissions to perform a command
    InsufficientPermissions,

    /// Invalid streaming id used
    InvalidStreamingId,

    /// Streaming target already in use
    StreamingTargetInUse,

    /// Recorder is not started
    RecorderNotStarted,

    /// Tried to start a recording for a room that has one already active
    RecordingAlreadyActive,

    /// Tried to pause or stop a recording, but it isn't active
    RecordingNotActive,

    /// Failed to request a recording service for this room
    FailedToRequestRecordingService,
}

impl ModuleError for RecordingError {}

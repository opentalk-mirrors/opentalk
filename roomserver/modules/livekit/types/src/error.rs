// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_roomserver_types::signaling::module_error::ModuleError;
use opentalk_types_signaling::ParticipantId;

/// Livekit errors
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum LiveKitError {
    /// Livekit server is not available
    LivekitUnavailable,

    /// Client has missing permissions to do the action on the livekit server
    InsufficientPermissions,

    /// The participant is not known.
    ///
    /// The participant might have disconnected before the command was executed.
    UnknownParticipant {
        /// A list of participants that are currently not part of the meeting.
        participant: BTreeSet<ParticipantId>,
    },
}

impl ModuleError for LiveKitError {}

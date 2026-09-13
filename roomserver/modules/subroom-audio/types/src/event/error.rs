// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types::signaling::module_error::ModuleError;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// Errors from the `subroom_audio` module namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "error")]
pub enum SubroomAudioError {
    /// The provided whisper id does not exist
    InvalidWhisperId,
    /// The participant has already accepted the group invite
    AlreadyAccepted,
    /// The requesting participant has insufficient permissions
    InsufficientPermissions,
    /// The list of invited participant is empty
    EmptyParticipantList,
    /// The targeted participants do not exist
    InvalidParticipantTargets {
        /// A list of invalid participants
        participant_ids: Vec<ParticipantId>,
    },
    /// The livekit server is unavailable
    LivekitUnavailable,
    /// The requesting participant has no access to the whisper group
    NotInvited,
}

impl ModuleError for SubroomAudioError {}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Peer frontend data for `meeting_notes` namespace
//!
//! These structs contain information about the module specific state of other participants in the
//! room and are send to a participant when they join a room.

use opentalk_types_signaling::SignalingModulePeerFrontendData;
use serde::{Deserialize, Serialize};

/// The state of other participants in the `meeting-notes` module.
///
/// This struct is sent to the participant in the `join_success` message
/// which will contain this information for each participant in the meeting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MeetingNotesPeerState {
    /// Read-only access
    pub readonly: bool,
}

impl SignalingModulePeerFrontendData for MeetingNotesPeerState {
    const NAMESPACE: Option<opentalk_types_common::modules::ModuleId> =
        Some(crate::MEETING_NOTES_MODULE_ID);
}

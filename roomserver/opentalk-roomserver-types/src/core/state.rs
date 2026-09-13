// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::{modules::ModuleId, time::Timestamp, users::DisplayName};
use opentalk_types_signaling::SignalingModulePeerFrontendData;
use serde::{Deserialize, Serialize};

use crate::{
    client_parameters::{ParticipationKind, Role},
    core::CORE_MODULE_ID,
};

/// The state of a participant in the `control` module.
///
/// This struct is sent to the participant in the `join_success` message
/// when they join successfully to the meeting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreState {
    /// Display name of the participant
    pub display_name: DisplayName,

    /// Role of the participant
    pub role: Role,

    /// The URL to the avatar of the participant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,

    /// The type of participant and how they connected to the meeting.
    pub participation_kind: ParticipationKind,

    /// The timestamp when the participant joined the meeting
    pub joined_at: Timestamp,

    /// The timestamp when the participant left the meeting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_at: Option<Timestamp>,

    /// Whether the participant is the room owner
    #[serde(default)]
    pub is_room_owner: bool,
}

impl SignalingModulePeerFrontendData for CoreState {
    const NAMESPACE: Option<ModuleId> = Some(CORE_MODULE_ID);
}

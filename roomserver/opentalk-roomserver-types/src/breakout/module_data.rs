// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::{modules::ModuleId, time::Timestamp};
use opentalk_types_signaling::{SignalingModuleFrontendData, SignalingModulePeerFrontendData};
use serde::{Deserialize, Serialize};

use crate::{breakout::BreakoutRoom, room_kind::RoomKind};

/// The module data that is attached to the `JoinSuccess` message
#[derive(Debug, Serialize, Deserialize)]
pub struct BreakoutModuleData {
    /// The current room of the participant
    pub room: RoomKind,

    /// Active breakout rooms
    pub rooms: Vec<BreakoutRoom>,

    /// The expiry for all breakout rooms
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires: Option<Timestamp>,
}

impl SignalingModuleFrontendData for BreakoutModuleData {
    const NAMESPACE: Option<ModuleId> = Some(super::BREAKOUT_MODULE_ID);
}

/// The peer module data that is attached to the `JoinSuccess` and `ParticipantJoined` messages.
#[derive(Debug, Serialize, Deserialize)]
pub struct BreakoutPeerModuleData {
    /// The current room of the participant
    pub room: RoomKind,
}

impl SignalingModulePeerFrontendData for BreakoutPeerModuleData {
    const NAMESPACE: Option<ModuleId> = Some(super::BREAKOUT_MODULE_ID);
}

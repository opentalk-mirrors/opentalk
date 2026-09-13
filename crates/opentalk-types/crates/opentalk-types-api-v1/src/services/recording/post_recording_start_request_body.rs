// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::rooms::{BreakoutRoomId, RoomId};

/// Request for the `POST /services/recording/start` endpoint
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PostRecordingStartRequestBody {
    /// The room id
    pub room_id: RoomId,

    /// The optional breakout room id
    pub breakout_room: Option<BreakoutRoomId>,
}

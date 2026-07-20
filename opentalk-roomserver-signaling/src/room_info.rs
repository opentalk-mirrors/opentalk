// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_types::room_parameters::RoomParameters;
use opentalk_types_common::{rooms::RoomId, time::Timestamp, users::UserId};

#[derive(Debug, Clone)]
pub struct RoomTaskInfo {
    /// the identifier of the room
    pub room_id: RoomId,
    /// The start parameters for the room task
    pub room: RoomParameters,
    /// The time at which the room will close
    pub closes_at: Option<Timestamp>,
}

impl RoomTaskInfo {
    pub fn owner(&self) -> UserId {
        self.room.created_by.id
    }

    pub fn mask_unmapped_phone_numbers(&self) -> Option<bool> {
        self.room
            .call_in
            .as_ref()
            .map(|call_in_info| call_in_info.mask_unmapped_phone_numbers)
    }

    pub fn default_country_code(&self) -> Option<&str> {
        self.room
            .call_in
            .as_ref()
            .map(|call_in_info| call_in_info.default_country_code.as_ref())
    }
}

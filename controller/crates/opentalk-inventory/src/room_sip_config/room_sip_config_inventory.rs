// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    call_in::CallInId,
    rooms::{RoomId, RoomIdOrAlias},
};

use super::{NewRoomSipConfig, RoomSipConfig, UpdateRoomSipConfig};
use crate::{Result, Room, User};

/// A trait for retrieving and storing room sip config entities.
#[async_trait::async_trait]
pub trait RoomSipConfigInventory {
    /// Get the SIP config for a room.
    async fn get_room_sip_config(&mut self, room_id: RoomId) -> Result<Option<RoomSipConfig>>;

    /// Get the SIP config, associated room and room creator for the provided call-in id.
    async fn get_room_sip_config_with_room_and_creator(
        &mut self,
        call_in_id: CallInId,
    ) -> Result<Option<(RoomSipConfig, Room, User)>>;

    /// Create a SIP config for a room
    async fn create_room_sip_config(
        &mut self,
        sip_config: NewRoomSipConfig,
    ) -> Result<RoomSipConfig>;

    /// Update a room SIP config.
    async fn update_room_sip_config(
        &mut self,
        room_id: RoomId,
        sip_config: UpdateRoomSipConfig,
    ) -> Result<Option<RoomSipConfig>>;

    /// Delete a room SIP config.
    async fn delete_room_sip_config(&mut self, room: &RoomIdOrAlias) -> Result<()>;
}

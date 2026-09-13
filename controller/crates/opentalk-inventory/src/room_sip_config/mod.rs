// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#![allow(clippy::module_inception)]

mod new_room_sip_config;
mod room_sip_config;
mod room_sip_config_inventory;
mod update_room_sip_config;

pub use new_room_sip_config::NewRoomSipConfig;
pub use room_sip_config::RoomSipConfig;
pub use room_sip_config_inventory::RoomSipConfigInventory;
pub use update_room_sip_config::UpdateRoomSipConfig;

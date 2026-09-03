// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#![allow(clippy::module_inception)]

mod room_invite;
mod room_invite_inventory;

pub use room_invite::RoomInvite;
#[cfg(feature = "mockall")]
pub use room_invite_inventory::MockRoomInviteInventory;
pub use room_invite_inventory::RoomInviteInventory;

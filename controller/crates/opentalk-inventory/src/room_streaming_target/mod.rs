// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod room_streaming_target_inventory;
mod room_streaming_target_record;
mod update_room_streaming_target;

#[cfg(feature = "mockall")]
pub use room_streaming_target_inventory::MockRoomStreamingTargetInventory;
pub use room_streaming_target_inventory::RoomStreamingTargetInventory;
pub use room_streaming_target_record::RoomStreamingTargetRecord;
pub use update_room_streaming_target::UpdateRoomStreamingTarget;

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains streaming targets table structs

mod new_room_streaming_target;
mod room_streaming_target;
mod update_room_streaming_target;

pub use new_room_streaming_target::NewRoomStreamingTarget;
pub use room_streaming_target::RoomStreamingTarget;
pub use update_room_streaming_target::UpdateRoomStreamingTarget;

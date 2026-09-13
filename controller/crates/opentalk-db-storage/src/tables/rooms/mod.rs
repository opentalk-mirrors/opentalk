// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains rooms table structs

mod new_room;
mod room;
mod serial_room_id;
mod update_room;

pub use new_room::NewRoom;
pub use room::Room;
pub use serial_room_id::SerialRoomId;
pub use update_room::UpdateRoom;

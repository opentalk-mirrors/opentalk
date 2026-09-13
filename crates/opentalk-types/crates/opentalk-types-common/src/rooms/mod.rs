// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Data types for handling rooms.

pub mod invite_codes;

mod breakout_room_id;
mod guest_access;
mod room_alias;
mod room_id;
mod room_id_or_alias;
mod room_name;
mod room_password;
mod room_suffix;

pub use breakout_room_id::BreakoutRoomId;
pub use guest_access::{GuestAccess, GuestAccessType};
pub use room_alias::{OptionalRoomAliasExt, RoomAlias};
pub use room_id::RoomId;
pub use room_id_or_alias::RoomIdOrAlias;
pub use room_name::{RoomName, TryFromRoomNameError};
pub use room_password::{ROOM_PASSWORD_MAX_LENGTH, ROOM_PASSWORD_MIN_LENGTH, RoomPassword};
pub use room_suffix::{InvalidCharacters, RoomSuffix};

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::rooms::{GuestAccess, RoomAlias, RoomPassword};

/// Representation of an update to a room in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateRoom {
    /// A custom room alias that can be used as part of the URL alias to access the room instead of the [`RoomId`].
    ///
    /// [`RoomId`]: opentalk_types_common::rooms::RoomId
    pub alias: Option<Option<RoomAlias>>,

    /// An optional password for the room.
    pub password: Option<Option<RoomPassword>>,

    /// A flag indicating that the wating room is enabled for this room.
    pub waiting_room: Option<bool>,

    /// Guest access mode for the room
    pub guest_access: Option<GuestAccess>,

    /// A flag indicating that e2e encryption is enabled for this room.
    pub e2e_encryption: Option<bool>,
}

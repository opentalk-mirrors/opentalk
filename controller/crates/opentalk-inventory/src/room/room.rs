// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    rooms::{GuestAccess, RoomAlias, RoomId, RoomPassword},
    tenants::TenantId,
    time::Timestamp,
    users::UserId,
};

/// The representation of a room in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Room {
    /// The id of the room.
    pub id: RoomId,

    /// The alias of the room.
    pub alias: Option<RoomAlias>,

    /// The serial id of the room.
    pub id_serial: i64,

    /// The id of the user who created the room.
    pub created_by: UserId,

    /// The creation timestamp.
    pub created_at: Timestamp,

    /// An optional password for the room.
    pub password: Option<RoomPassword>,

    /// A flag indicating that the wating room is enabled for this room.
    pub waiting_room: bool,

    /// Guest access mode for the room
    pub guest_access: GuestAccess,

    /// The id of the tenant to which the room belongs.
    pub tenant_id: TenantId,

    /// A flag indicating that e2e encryption is enabled for this room.
    pub e2e_encryption: bool,
}

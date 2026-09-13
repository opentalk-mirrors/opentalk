// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    rooms::{GuestAccess, RoomAlias, RoomPassword},
    tenants::TenantId,
    users::UserId,
};

/// The representation of a new room that is intended to be stored in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewRoom {
    /// The creator of the room.
    pub created_by: UserId,

    /// An alias that can be used in the URL to access the room instead of the [`RoomId`](opentalk_types_common::rooms::RoomId).
    pub alias: Option<RoomAlias>,

    /// An optional password for the room.
    pub password: Option<RoomPassword>,

    /// A flag indicating whether the waiting room is active.
    pub waiting_room: bool,

    /// Guest access mode for the room
    pub guest_access: GuestAccess,

    /// The id of the tenant to which the room belongs.
    pub tenant_id: TenantId,

    /// A flag indicating whether the e2e encryption is enabled for the room.
    pub e2e_encryption: bool,
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_inventory as inventory;
use opentalk_types_common::{
    rooms::{GuestAccess, OptionalRoomAliasExt as _, RoomName, RoomPassword, RoomSuffix},
    tenants::TenantId,
    users::UserId,
};

use crate::schema::rooms;

/// Diesel insertable room struct
///
/// Represents fields that have to be provided on room insertion.
#[derive(Debug, Insertable)]
#[diesel(table_name = rooms)]
pub struct NewRoom {
    pub created_by: UserId,
    pub password: Option<RoomPassword>,
    pub waiting_room: bool,
    pub guest_access: GuestAccess,
    pub tenant_id: TenantId,
    pub e2e_encryption: bool,
    pub name: Option<RoomName>,
    pub suffix: Option<RoomSuffix>,
}

impl From<inventory::NewRoom> for NewRoom {
    fn from(
        inventory::NewRoom {
            created_by,
            alias,
            password,
            waiting_room,
            guest_access,
            tenant_id,
            e2e_encryption,
        }: inventory::NewRoom,
    ) -> Self {
        let (name, suffix) = alias.into_parts();

        Self {
            created_by,
            password,
            waiting_room,
            guest_access,
            tenant_id,
            e2e_encryption,
            name,
            suffix,
        }
    }
}

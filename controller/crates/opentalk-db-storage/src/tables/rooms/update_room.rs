// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_inventory as inventory;
use opentalk_types_common::rooms::{
    GuestAccess, OptionalRoomAliasExt, RoomName, RoomPassword, RoomSuffix,
};

use crate::schema::rooms;

/// Diesel room struct for updates
///
/// Is used in update queries. None fields will be ignored on update queries
#[derive(Debug, AsChangeset)]
#[diesel(table_name = rooms)]
pub struct UpdateRoom {
    pub name: Option<Option<RoomName>>,
    pub suffix: Option<Option<RoomSuffix>>,
    pub password: Option<Option<RoomPassword>>,
    pub waiting_room: Option<bool>,
    pub guest_access: Option<GuestAccess>,
    pub e2e_encryption: Option<bool>,
}

impl From<inventory::UpdateRoom> for UpdateRoom {
    fn from(
        inventory::UpdateRoom {
            alias,
            password,
            waiting_room,
            guest_access,
            e2e_encryption,
        }: inventory::UpdateRoom,
    ) -> Self {
        let (name, suffix) = if let Some(alias) = alias {
            let (name, suffix) = alias.into_parts();
            (Some(name), Some(suffix))
        } else {
            (None, None)
        };

        Self {
            name,
            suffix,
            password,
            waiting_room,
            guest_access,
            e2e_encryption,
        }
    }
}

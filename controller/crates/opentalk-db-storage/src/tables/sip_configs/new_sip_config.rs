// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::prelude::*;
use opentalk_inventory as inventory;
use opentalk_types_common::{
    call_in::{CallInId, CallInPassword},
    rooms::RoomId,
};

use crate::schema::sip_configs;

/// Diesel insertable SipConfig struct
///
/// Represents fields that have to be provided on insertion.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = sip_configs)]
pub struct NewSipConfig {
    pub room: RoomId,
    pub sip_id: CallInId,
    pub password: CallInPassword,
    pub enable_lobby: bool,
}

impl From<inventory::NewRoomSipConfig> for NewSipConfig {
    fn from(
        inventory::NewRoomSipConfig {
            room,
            sip_id,
            password,
            enable_lobby,
        }: inventory::NewRoomSipConfig,
    ) -> Self {
        Self {
            room,
            sip_id,
            password,
            enable_lobby,
        }
    }
}

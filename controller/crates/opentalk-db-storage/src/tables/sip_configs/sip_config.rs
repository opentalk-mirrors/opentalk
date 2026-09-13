// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::{Identifiable, Queryable};
use opentalk_inventory as inventory;
use opentalk_types_common::{
    call_in::{CallInId, CallInPassword},
    rooms::RoomId,
};

use crate::schema::sip_configs;

/// Diesel SipConfig struct
#[derive(Debug, Clone, Queryable, Identifiable)]
#[diesel(table_name = sip_configs)]
pub struct SipConfig {
    pub id: i64,
    pub room: RoomId,
    pub sip_id: CallInId,
    pub password: CallInPassword,
    pub lobby: bool,
}

impl From<SipConfig> for inventory::RoomSipConfig {
    fn from(
        SipConfig {
            id,
            room,
            sip_id,
            password,
            lobby,
        }: SipConfig,
    ) -> Self {
        Self {
            id,
            room,
            sip_id,
            password,
            lobby,
        }
    }
}

impl From<inventory::RoomSipConfig> for SipConfig {
    fn from(
        inventory::RoomSipConfig {
            id,
            room,
            sip_id,
            password,
            lobby,
        }: inventory::RoomSipConfig,
    ) -> Self {
        Self {
            id,
            room,
            sip_id,
            password,
            lobby,
        }
    }
}

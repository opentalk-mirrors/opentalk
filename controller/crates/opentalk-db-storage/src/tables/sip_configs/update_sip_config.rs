// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::prelude::*;
use opentalk_inventory as inventory;
use opentalk_types_common::call_in::CallInPassword;

use crate::schema::sip_configs;

/// Diesel struct to modify a SipConfig
#[derive(Debug, AsChangeset)]
#[diesel(table_name = sip_configs)]
pub struct UpdateSipConfig {
    pub password: Option<CallInPassword>,
    pub enable_lobby: Option<bool>,
}

impl From<inventory::UpdateRoomSipConfig> for UpdateSipConfig {
    fn from(
        inventory::UpdateRoomSipConfig {
            password,
            enable_lobby,
        }: inventory::UpdateRoomSipConfig,
    ) -> Self {
        Self {
            password,
            enable_lobby,
        }
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::Insertable;
use opentalk_types_common::{assets::AssetId, rooms::RoomId};

use crate::schema::room_assets;

#[derive(Debug, Insertable)]
#[diesel(table_name = room_assets)]
pub struct RoomAsset {
    pub room_id: RoomId,
    pub asset_id: AssetId,
}

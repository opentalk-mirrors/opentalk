// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_inventory as inventory;

use crate::schema::assets;

#[derive(Debug, AsChangeset)]
#[diesel(table_name = assets)]
pub struct UpdateAsset {
    pub size: Option<i64>,
    pub filename: Option<String>,
}

impl From<UpdateAsset> for inventory::UpdateAsset {
    fn from(UpdateAsset { size, filename }: UpdateAsset) -> Self {
        Self { size, filename }
    }
}

impl From<inventory::UpdateAsset> for UpdateAsset {
    fn from(inventory::UpdateAsset { size, filename }: inventory::UpdateAsset) -> Self {
        Self { size, filename }
    }
}

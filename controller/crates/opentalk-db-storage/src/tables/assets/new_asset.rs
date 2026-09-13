// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::Insertable;
use opentalk_inventory as inventory;
use opentalk_types_common::{assets::AssetId, modules::ModuleId, tenants::TenantId};

use crate::schema::assets;

#[derive(Debug, Insertable)]
#[diesel(table_name = assets)]
pub struct NewAsset {
    pub id: AssetId,
    pub namespace: Option<ModuleId>,
    pub kind: String,
    pub filename: String,
    pub tenant_id: TenantId,
    pub size: i64,
}

impl From<inventory::NewAsset> for NewAsset {
    fn from(
        inventory::NewAsset {
            id,
            namespace,
            kind,
            filename,
            tenant_id,
            size,
        }: inventory::NewAsset,
    ) -> Self {
        Self {
            id,
            namespace,
            kind,
            filename,
            tenant_id,
            size,
        }
    }
}

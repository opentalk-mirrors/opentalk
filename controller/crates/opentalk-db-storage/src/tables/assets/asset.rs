// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use diesel::{Identifiable, Queryable};
use opentalk_inventory as inventory;
use opentalk_types_common::{
    assets::{AssetId, FileSize},
    modules::ModuleId,
    tenants::TenantId,
};

use crate::schema::assets;

/// Diesel resource struct
#[derive(Debug, Clone, Queryable, Identifiable)]
#[diesel(table_name = assets)]
pub struct Asset {
    pub id: AssetId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub namespace: Option<ModuleId>,
    pub kind: String,
    pub filename: String,
    pub tenant_id: TenantId,
    pub size: FileSize,
}

impl From<Asset> for inventory::Asset {
    fn from(
        Asset {
            id,
            created_at,
            updated_at,
            namespace,
            kind,
            filename,
            tenant_id,
            size,
        }: Asset,
    ) -> Self {
        Self {
            id,
            created_at: created_at.into(),
            updated_at: updated_at.into(),
            namespace,
            kind,
            filename,
            tenant_id,
            size,
        }
    }
}

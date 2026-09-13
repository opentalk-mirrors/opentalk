// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use diesel::{Identifiable, Queryable};
use opentalk_inventory as inventory;
use opentalk_types_common::{
    module_resources::ModuleResourceId, rooms::RoomId, tenants::TenantId, users::UserId,
};

use crate::schema::module_resources;

#[derive(Debug, Clone, Eq, PartialEq, Queryable, Identifiable, Insertable)]
#[diesel(table_name = module_resources)]
pub struct ModuleResource {
    pub id: ModuleResourceId,
    pub tenant_id: TenantId,
    pub room_id: RoomId,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub namespace: String,
    pub tag: Option<String>,
    pub data: serde_json::Value,
}

impl From<ModuleResource> for inventory::ModuleResource {
    fn from(
        ModuleResource {
            id,
            tenant_id,
            room_id,
            created_by,
            created_at,
            updated_at,
            namespace,
            tag,
            data,
        }: ModuleResource,
    ) -> Self {
        Self {
            id,
            tenant_id,
            room_id,
            created_by,
            created_at: created_at.into(),
            updated_at: updated_at.into(),
            namespace,
            tag,
            data,
        }
    }
}

impl From<inventory::ModuleResource> for ModuleResource {
    fn from(
        inventory::ModuleResource {
            id,
            tenant_id,
            room_id,
            created_by,
            created_at,
            updated_at,
            namespace,
            tag,
            data,
        }: inventory::ModuleResource,
    ) -> Self {
        Self {
            id,
            tenant_id,
            room_id,
            created_by,
            created_at: created_at.into(),
            updated_at: updated_at.into(),
            namespace,
            tag,
            data,
        }
    }
}

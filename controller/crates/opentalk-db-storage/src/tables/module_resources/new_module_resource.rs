// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::Insertable;
use opentalk_inventory as inventory;
use opentalk_types_common::{rooms::RoomId, tenants::TenantId, users::UserId};

use crate::schema::module_resources;

#[derive(Debug, Insertable)]
#[diesel(table_name = module_resources)]
pub struct NewModuleResource {
    pub tenant_id: TenantId,
    pub room_id: RoomId,
    pub created_by: UserId,
    pub namespace: String,
    pub tag: Option<String>,
    pub data: serde_json::Value,
}

impl From<inventory::NewModuleResource> for NewModuleResource {
    fn from(
        inventory::NewModuleResource {
            tenant_id,
            room_id,
            created_by,
            namespace,
            tag,
            data,
        }: inventory::NewModuleResource,
    ) -> Self {
        Self {
            tenant_id,
            room_id,
            created_by,
            namespace,
            tag,
            data,
        }
    }
}

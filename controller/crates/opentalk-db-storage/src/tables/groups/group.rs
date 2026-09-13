// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::prelude::*;
use opentalk_inventory as inventory;
use opentalk_types_common::{
    tenants::TenantId,
    users::{GroupId, GroupName},
};

use crate::{schema::groups, tables::groups::SerialGroupId};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Queryable, Insertable, Identifiable)]
#[diesel(table_name = groups)]
pub struct Group {
    pub id: GroupId,
    pub id_serial: SerialGroupId,
    pub name: GroupName,
    pub tenant_id: TenantId,
}

impl From<Group> for inventory::Group {
    fn from(
        Group {
            id,
            id_serial: _,
            name,
            tenant_id,
        }: Group,
    ) -> Self {
        Self {
            id,
            name,
            tenant_id,
        }
    }
}

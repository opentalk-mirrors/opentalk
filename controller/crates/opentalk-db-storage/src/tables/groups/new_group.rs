// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::prelude::*;
use opentalk_types_common::{tenants::TenantId, users::GroupName};

use crate::schema::groups;

#[derive(Debug, Insertable)]
#[diesel(table_name = groups)]
pub struct NewGroup<'a> {
    pub name: &'a GroupName,
    pub tenant_id: TenantId,
}

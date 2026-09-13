// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use diesel::prelude::*;

use crate::{schema::tenants, tables::tenants::OidcTenantId};

#[derive(AsChangeset)]
#[diesel(table_name = tenants)]
pub struct UpdateTenant<'a> {
    pub updated_at: DateTime<Utc>,
    pub oidc_tenant_id: &'a OidcTenantId,
}

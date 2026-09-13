// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::prelude::*;

use crate::{schema::tenants, tables::tenants::OidcTenantId};

#[derive(Clone, Insertable)]
#[diesel(table_name = tenants)]
pub struct NewTenant<'n> {
    pub oidc_tenant_id: &'n OidcTenantId,
}

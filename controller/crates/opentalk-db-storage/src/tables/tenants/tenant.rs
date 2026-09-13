// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use opentalk_inventory as inventory;
use opentalk_types_common::tenants::TenantId;
use serde::{Deserialize, Serialize};

use crate::{schema::tenants, tables::tenants::OidcTenantId};

#[derive(Debug, Clone, Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = tenants)]
pub struct Tenant {
    pub id: TenantId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub oidc_tenant_id: OidcTenantId,
}

impl From<Tenant> for inventory::Tenant {
    fn from(
        Tenant {
            id,
            created_at,
            updated_at,
            oidc_tenant_id,
        }: Tenant,
    ) -> Self {
        Self {
            id,
            created_at: created_at.into(),
            updated_at: updated_at.into(),
            oidc_tenant_id: oidc_tenant_id.into(),
        }
    }
}

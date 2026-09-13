// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{tenants::TenantId, time::Timestamp, utils::ExampleData};

use super::OidcTenantId;

/// Information about a tenant stored in the inventory.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Tenant {
    /// The id of the tenant.
    pub id: TenantId,

    /// The creation timestamp.
    pub created_at: Timestamp,

    /// The updated timestamp.
    pub updated_at: Timestamp,

    /// The OIDC tenant id.
    pub oidc_tenant_id: OidcTenantId,
}

impl ExampleData for Tenant {
    fn example_data() -> Self {
        Self {
            id: TenantId::nil(),
            created_at: Timestamp::unix_epoch(),
            updated_at: Timestamp::unix_epoch(),
            oidc_tenant_id: OidcTenantId::from("tenant-id"),
        }
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::tenants::TenantId;

use super::{OidcTenantId, Tenant};
use crate::Result;

/// A trait for retrieving and storing room entities.
#[async_trait::async_trait]
pub trait TenantInventory {
    /// Get a tenant by its id.
    async fn get_tenant(&mut self, tenant_id: TenantId) -> Result<Tenant>;

    /// Get a tenant by the [`OidcTenantId`], or create a new one if it doesn't exist yet.
    async fn get_or_create_tenant_by_oidc_id(
        &mut self,
        oidc_tenant_id: &OidcTenantId,
    ) -> Result<Tenant>;
}

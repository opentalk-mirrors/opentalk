// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage as db;
use opentalk_inventory::{OidcTenantId, Tenant, TenantInventory};
use opentalk_types_common::tenants::TenantId;
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result, error::DatabaseSnafu};

#[async_trait::async_trait]
impl TenantInventory for DatabaseConnection {
    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_tenant(&mut self, tenant_id: TenantId) -> Result<Tenant> {
        Ok(db::queries::tenants::get_tenant(&mut self.inner, tenant_id)
            .await
            .context(DatabaseSnafu)?
            .into())
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_or_create_tenant_by_oidc_id(
        &mut self,
        oidc_tenant_id: &OidcTenantId,
    ) -> Result<Tenant> {
        Ok(db::queries::tenants::get_or_create_tenant_by_oidc_id(
            &mut self.inner,
            &oidc_tenant_id.into(),
        )
        .await
        .context(DatabaseSnafu)?
        .into())
    }
}

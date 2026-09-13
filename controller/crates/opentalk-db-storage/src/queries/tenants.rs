// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains tenantts database queries
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use opentalk_database::{DatabaseError, DbConnection, Result};
use opentalk_types_common::tenants::TenantId;

use crate::{
    schema::tenants,
    tables::tenants::{NewTenant, OidcTenantId, Tenant, UpdateTenant},
};

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_tenant(conn: &mut DbConnection, id: TenantId) -> Result<Tenant> {
    tenants::table
        .filter(tenants::id.eq(id))
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

pub async fn get_all_tenants(conn: &mut DbConnection) -> Result<Vec<Tenant>> {
    tenants::table.load(conn).await.map_err(DatabaseError::from)
}

/// Get or create a tenant by name
pub async fn get_or_create_tenant_by_oidc_id(
    conn: &mut DbConnection,
    oidc_tenant_id: &OidcTenantId,
) -> Result<Tenant> {
    let maybe_tenant = tenants::table
        .select(tenants::all_columns)
        .filter(tenants::oidc_tenant_id.eq(oidc_tenant_id))
        .get_result(conn)
        .await
        .optional()?;

    let Some(tenant) = maybe_tenant else {
        let new_tenant = NewTenant { oidc_tenant_id };

        return diesel::insert_into(tenants::table)
            .values(new_tenant)
            .returning(tenants::all_columns)
            .get_result(conn)
            .await
            .map_err(DatabaseError::from);
    };

    Ok(tenant)
}

pub async fn update_tenant(
    conn: &mut DbConnection,
    update_tenant: UpdateTenant<'_>,
    tenant_id: TenantId,
) -> Result<Tenant> {
    diesel::update(tenants::table.filter(tenants::id.eq(tenant_id)))
        .set(update_tenant)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

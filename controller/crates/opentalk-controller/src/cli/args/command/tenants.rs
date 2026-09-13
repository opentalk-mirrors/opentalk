// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::path::Path;

use chrono::Utc;
use clap::Subcommand;
use opentalk_controller_core::load_settings_provider;
use opentalk_controller_settings::Settings;
use opentalk_database::{DatabaseError, Db};
use opentalk_db_storage as db;
use opentalk_inventory::OidcTenantId;
use opentalk_types_common::tenants::TenantId;
use snafu::ResultExt as _;
use tabled::{Table, Tabled, settings::Style};
use uuid::Uuid;

use crate::Result;

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Command {
    /// List all available tenants
    List,
    /// Change a tenants oidc-id
    SetOidcId { id: Uuid, new_oidc_id: String },
}

impl Command {
    pub async fn exec(self, optional_config_path: Option<&Path>) -> Result<()> {
        let settings = load_settings_provider(optional_config_path)?.get();
        match self {
            Command::List => list_all_tenants(&settings).await,
            Command::SetOidcId { id, new_oidc_id } => {
                set_oidc_id(
                    &settings,
                    TenantId::from(id),
                    OidcTenantId::from(new_oidc_id),
                )
                .await
            }
        }
        .whatever_context("Tenants command failed")
    }
}

#[derive(Tabled)]
struct TenantTableRow {
    id: TenantId,
    oidc_id: OidcTenantId,
}

impl TenantTableRow {
    fn from_tenant(tenant: db::tables::tenants::Tenant) -> Self {
        Self {
            id: tenant.id,
            oidc_id: tenant.oidc_tenant_id.into(),
        }
    }
}

/// Implementation of the `opentalk-controller tenants list` command
async fn list_all_tenants(settings: &Settings) -> Result<(), DatabaseError> {
    let db = Db::connect(&settings.database)?;
    let mut conn = db.get_conn().await?;

    let tenants = db::queries::tenants::get_all_tenants(&mut conn).await?;
    let rows: Vec<TenantTableRow> = tenants
        .into_iter()
        .map(TenantTableRow::from_tenant)
        .collect();

    println!("{}", Table::new(rows).with(Style::psql()));

    Ok(())
}

/// Implementation of the `opentalk-controller tenants set-oidc-id <tenant-id> <new-oidc-id>` command
async fn set_oidc_id(
    settings: &Settings,
    id: TenantId,
    new_oidc_id: OidcTenantId,
) -> Result<(), DatabaseError> {
    let db = Db::connect(&settings.database)?;
    let mut conn = db.get_conn().await?;

    let tenant = db::queries::tenants::get_tenant(&mut conn, id).await?;
    let old_oidc_id = tenant.oidc_tenant_id;

    let update_tenant = db::tables::tenants::UpdateTenant {
        updated_at: Utc::now(),
        oidc_tenant_id: &new_oidc_id.clone().into(),
    };

    let _ = db::queries::tenants::update_tenant(&mut conn, update_tenant, id).await?;

    println!(
        "Updated tenant's oidc-id\n\tid  = {id}\n\told = {old_oidc_id}\n\tnew = {new_oidc_id}"
    );

    Ok(())
}

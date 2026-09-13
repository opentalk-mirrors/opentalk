// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    str::FromStr,
    sync::Arc,
};

use clap::Subcommand;
use humansize::{DECIMAL, FormatSizeOptions, format_size};
use itertools::Itertools;
use opentalk_controller_core::load_settings_provider;
use opentalk_controller_settings::Settings;
use opentalk_database::{DatabaseError, Db};
use opentalk_inventory::{
    ExternalTariffId, Inventory, InventoryProvider as _, NewTariff, Tariff, UpdateTariff,
    transaction,
};
use opentalk_inventory_database::DatabaseConnectionPool;
use opentalk_types_common::{
    features::ModuleFeatureId, modules::ModuleId, tariffs::QuotaType, time::Timestamp,
};
use parse_size::parse_size;
use snafu::{OptionExt, ResultExt, Snafu};
use tabled::{Table, Tabled, settings::Style};

use crate::Result;

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Command {
    /// List all available tariffs
    List,
    /// Create a new tariff
    Create {
        /// Name of the tariff
        tariff_name: String,
        /// Eternal ID to map to the tariff
        external_tariff_id: String,
        /// Comma-separated list of modules to disable
        #[clap(long, value_delimiter = ',')]
        disabled_modules: Vec<ModuleId>,
        /// Comma-separated list of features to disable
        #[clap(long, value_delimiter = ',')]
        disabled_features: Vec<ModuleFeatureId>,
        /// Comma-separated list of key=value pairs
        #[clap(long, value_delimiter = ',', value_parser = parse_quota)]
        quotas: Vec<(QuotaType, u64)>,
    },
    /// Delete a tariff by name
    Delete {
        /// Name of the tariff to delete
        tariff_name: String,
    },

    /// Modify an existing tariff
    Edit {
        /// Name of the tariff to modify
        tariff_name: String,

        /// Set a new name
        #[clap(long)]
        set_name: Option<String>,

        /// Comma-separated list of external tariff_ids to add
        #[clap(long, value_delimiter = ',')]
        add_external_tariff_ids: Vec<String>,

        /// Comma-separated list of external tariff_ids to remove
        #[clap(long, value_delimiter = ',')]
        remove_external_tariff_ids: Vec<String>,

        /// Comma-separated list of module names to add
        #[clap(long, value_delimiter = ',')]
        add_disabled_modules: Vec<ModuleId>,

        /// Comma-separated list of module names to remove
        #[clap(long, value_delimiter = ',')]
        remove_disabled_modules: Vec<ModuleId>,

        /// Comma-separated list of feature names to add
        #[clap(long, value_delimiter = ',')]
        add_disabled_features: Vec<ModuleFeatureId>,

        /// Comma-separated list of feature names to remove
        #[clap(long, value_delimiter = ',')]
        remove_disabled_features: Vec<ModuleFeatureId>,

        /// Comma-separated list of key=value pairs to add, overwrites quotas with the same name
        #[clap(long, value_delimiter = ',', value_parser = parse_quota)]
        add_quotas: Vec<(QuotaType, u64)>,

        /// Comma-separated list of quota keys to remove
        #[clap(long, value_delimiter = ',')]
        remove_quotas: Vec<QuotaType>,
    },
}

impl Command {
    pub async fn exec(self, optional_config_path: Option<&Path>) -> Result<()> {
        let settings = load_settings_provider(optional_config_path)?.get();
        match self {
            Command::List => list_all_tariffs(&settings).await,
            Command::Create {
                tariff_name,
                external_tariff_id,
                disabled_modules,
                disabled_features,
                quotas,
            } => {
                create_tariff(
                    &settings,
                    tariff_name,
                    external_tariff_id,
                    BTreeSet::from_iter(disabled_modules),
                    BTreeSet::from_iter(disabled_features),
                    quotas.into_iter().collect(),
                )
                .await
            }
            Command::Delete { tariff_name } => delete_tariff(&settings, tariff_name).await,
            Command::Edit {
                tariff_name,
                set_name,
                add_external_tariff_ids,
                remove_external_tariff_ids,
                add_disabled_modules,
                remove_disabled_modules,
                add_disabled_features,
                remove_disabled_features,
                add_quotas,
                remove_quotas,
            } => {
                edit_tariff(
                    &settings,
                    tariff_name,
                    set_name,
                    add_external_tariff_ids,
                    remove_external_tariff_ids,
                    BTreeSet::from_iter(add_disabled_modules),
                    BTreeSet::from_iter(remove_disabled_modules),
                    BTreeSet::from_iter(add_disabled_features),
                    BTreeSet::from_iter(remove_disabled_features),
                    add_quotas.into_iter().collect(),
                    remove_quotas,
                )
                .await
            }
        }
        .whatever_context("Tariffs command failed")
    }
}

#[derive(Debug, Snafu)]
enum CliParameterError {
    /// Invalid key-value-pair, must be of form `key=value`
    KeyValuePair,
    /// invalid quota type
    QuotaType { source: strum::ParseError },
    /// invalid quota value, expected 64-bit unsigned integer or size like 1M, 1MB, 1Mi, 1MiB, 1e3 or similar
    QuotaValue { source: parse_size::Error },
}

fn parse_quota(s: &str) -> Result<(QuotaType, u64), CliParameterError> {
    let (name, value) = s.split_once('=').context(KeyValuePairSnafu)?;
    let value = parse_size(value.trim()).context(QuotaValueSnafu)?;
    Ok((QuotaType::from_str(name).context(QuotaTypeSnafu)?, value))
}

#[derive(Debug, Snafu)]
pub enum CliExecutionError {
    /// Database error
    #[snafu(transparent)]
    Database { source: DatabaseError },

    /// Diesel error
    #[snafu(transparent)]
    Diesel { source: diesel::result::Error },

    /// Inventory error
    #[snafu(transparent)]
    Inventory { source: opentalk_inventory::Error },
}

async fn list_all_tariffs(settings: &Settings) -> Result<(), CliExecutionError> {
    let db = Arc::new(Db::connect(&settings.database)?);
    let inventory_provider = DatabaseConnectionPool::new(db);
    let mut inventory = inventory_provider.get_inventory().await?;

    let tariffs = inventory.get_all_tariffs().await?;

    print_tariffs(inventory.as_mut(), tariffs).await
}

async fn create_tariff(
    settings: &Settings,
    name: String,
    external_tariff_id: String,
    disabled_modules: BTreeSet<ModuleId>,
    disabled_features: BTreeSet<ModuleFeatureId>,
    quotas: BTreeMap<QuotaType, u64>,
) -> Result<(), CliExecutionError> {
    let db = Arc::new(Db::connect(&settings.database)?);
    let inventory_provider = DatabaseConnectionPool::new(db);
    let mut inventory = inventory_provider.get_inventory().await?;

    transaction(inventory.as_mut(), async |inventory| {
        let tariff = inventory.create_tariff(

        NewTariff {
            name: name.clone(),
            quotas,
            disabled_modules,
            disabled_features,
        }).await?;

        inventory.create_external_tariff_mapping(external_tariff_id.clone().into(), tariff.id).await?;


        println!(
            "Created tariff name={name:?} with external external_tariff_id={external_tariff_id:?} ({})",
            tariff.id
        );

        Ok(())
    })
    .await
}

async fn delete_tariff(settings: &Settings, name: String) -> Result<(), CliExecutionError> {
    let db = Arc::new(Db::connect(&settings.database)?);
    let inventory_provider = DatabaseConnectionPool::new(db);
    let mut inventory = inventory_provider.get_inventory().await?;

    transaction(inventory.as_mut(), async |inventory| {
        let tariff = inventory.get_tariff_by_name(&name).await?;
        inventory
            .delete_all_external_tariff_mappings_for_tariff(tariff.id)
            .await?;
        inventory.delete_tariff(tariff.id).await?;

        println!("Deleted tariff name={name:?} ({})", tariff.id);

        Ok(())
    })
    .await
}

#[expect(clippy::too_many_arguments)]
async fn edit_tariff(
    settings: &Settings,
    name: String,
    set_name: Option<String>,
    add_external_tariff_ids: Vec<String>,
    remove_external_tariff_ids: Vec<String>,
    add_disabled_modules: BTreeSet<ModuleId>,
    remove_disabled_modules: BTreeSet<ModuleId>,
    add_disabled_features: BTreeSet<ModuleFeatureId>,
    remove_disabled_features: BTreeSet<ModuleFeatureId>,
    add_quotas: BTreeMap<QuotaType, u64>,
    remove_quotas: Vec<QuotaType>,
) -> Result<(), CliExecutionError> {
    let db = Arc::new(Db::connect(&settings.database)?);
    let inventory_provider = DatabaseConnectionPool::new(db);
    let mut inventory = inventory_provider.get_inventory().await?;

    transaction(inventory.as_mut(), async |inventory| {
        let tariff = inventory.get_tariff_by_name(&name).await?;

        // Remove all specified external tariff ids
        if !remove_external_tariff_ids.is_empty() {
            let external_tariff_ids_to_remove: Vec<ExternalTariffId> = remove_external_tariff_ids
                .into_iter()
                .map(ExternalTariffId::from)
                .collect();
            inventory
                .delete_external_tariff_mappings_for_tariff_by_external_id(
                    tariff.id,
                    &external_tariff_ids_to_remove,
                )
                .await?;
        }

        // Add all specified external tariff ids
        if !add_external_tariff_ids.is_empty() {
            for to_add in add_external_tariff_ids {
                inventory
                    .create_external_tariff_mapping(to_add.into(), tariff.id)
                    .await?;
            }
        }

        // Modify the `disabled_modules` list
        let mut disabled_modules = tariff.disabled_modules();
        disabled_modules
            .retain(|disabled_module| !remove_disabled_modules.contains(disabled_module));
        disabled_modules.extend(add_disabled_modules);

        // Modify the `disabled_features` list
        let mut disabled_features = tariff.disabled_features();
        disabled_features
            .retain(|disabled_module| !remove_disabled_features.contains(disabled_module));
        disabled_features.extend(add_disabled_features);

        // Modify the `quotas` set
        let mut quotas = tariff.quotas.clone();
        quotas.retain(|key, _| !remove_quotas.contains(key));
        quotas.extend(add_quotas);

        // Apply changeset
        let updated_tariff = inventory
            .update_tariff(
                tariff,
                UpdateTariff {
                    name: set_name,
                    updated_at: Timestamp::now(),
                    quotas: Some(quotas),
                    disabled_modules: Some(Vec::from_iter(disabled_modules)),
                    disabled_features: Some(Vec::from_iter(disabled_features)),
                },
            )
            .await?;

        println!(
            "Updated tariff name={:?} ({})",
            updated_tariff.name, updated_tariff.id
        );
        print_tariffs(inventory, [updated_tariff]).await?;
        Ok(())
    })
    .await
}

/// Print the list of tariffs as table
async fn print_tariffs(
    inventory: &mut dyn Inventory,
    tariffs: impl IntoIterator<Item = Tariff>,
) -> Result<(), CliExecutionError> {
    #[derive(Tabled)]
    struct TariffTableRow {
        #[tabled(rename = "name (internal)")]
        name: String,
        #[tabled(rename = "external tariff_id")]
        ext: String,
        #[tabled(rename = "disabled modules")]
        disabled_modules: String,
        #[tabled(rename = "disabled features")]
        disabled_features: String,
        quotas: String,
    }

    let mut rows = vec![];

    for tariff in tariffs {
        let ids = inventory
            .get_all_external_tariff_ids_for_tariff(tariff.id)
            .await?;
        let mut ids = ids
            .into_iter()
            .map(|ext_tariff_id| ext_tariff_id.to_string())
            .join("\n");
        if ids.is_empty() {
            ids = "-".into();
        }

        let mut disabled_modules = tariff.disabled_modules().into_iter().join("\n");
        if disabled_modules.is_empty() {
            disabled_modules = "-".into();
        }

        let mut disabled_features = tariff.disabled_features().into_iter().join("\n");
        if disabled_features.is_empty() {
            disabled_features = "-".into();
        }

        let mut quotas = tariff
            .quotas
            .into_iter()
            .map(|(k, v)| {
                if matches!(k, QuotaType::MaxStorage) {
                    let options = FormatSizeOptions::from(DECIMAL).decimal_zeroes(2);
                    let hrv = format_size(v, options);
                    format!("{k}: {v} ({hrv})")
                } else {
                    format!("{k}: {v}")
                }
            })
            .join("\n");
        if quotas.is_empty() {
            quotas = "-".into();
        }

        rows.push(TariffTableRow {
            name: tariff.name,
            ext: ids,
            disabled_modules,
            disabled_features,
            quotas,
        });
    }

    println!("{}", Table::new(rows).with(Style::ascii()));

    Ok(())
}

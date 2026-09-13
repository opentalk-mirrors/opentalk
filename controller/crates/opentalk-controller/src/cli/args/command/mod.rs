// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::path::Path;

use clap::Subcommand;

use crate::Result;

mod acl;
mod dev;
mod fix_acl;
mod health;
mod jobs;
mod migrate_db;
mod modules;
mod openapi;
pub(super) mod reload;
mod tariffs;
mod tenants;

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Command {
    /// Migrate the db. This is done automatically during start of the controller,
    /// but can be done without starting the controller using this command.
    MigrateDb(migrate_db::Command),

    /// Manage existing tenants
    #[clap(subcommand)]
    Tenants(tenants::Command),

    /// Manage tariffs
    #[clap(subcommand)]
    Tariffs(tariffs::Command),

    /// Manage and execute maintenance jobs
    #[clap(subcommand)]
    Jobs(jobs::Command),

    /// Manage modules
    #[clap(subcommand)]
    Modules(modules::Command),

    /// Get information on the OpenAPI specification
    #[clap(subcommand)]
    Openapi(openapi::Command),

    /// Return the readiness state
    Health(health::Command),

    /// Triggers a reload of reloadable configuration options for already
    /// running opentalk-controller processes
    Reload(reload::Command),

    /// The `fix-acl` is no longer in use, but we still want to tell the users
    /// who were trained to use it that it is no longer necessary.
    #[clap(hide = true)]
    FixAcl(fix_acl::Command),

    /// The `acl` is no longer in use, but we still want to tell the users
    /// who are attempting to use it.
    #[clap(hide = true)]
    Acl(acl::Command),

    /// Development and debugging helpers
    #[clap(subcommand, hide = true)]
    Dev(dev::Command),
}

impl Command {
    pub async fn exec(self, optional_config_path: Option<&Path>) -> Result<()> {
        match self {
            Command::MigrateDb(command) => {
                command.exec(optional_config_path).await?;
            }
            Command::Tenants(command) => {
                command.exec(optional_config_path).await?;
            }
            Command::Tariffs(command) => {
                command.exec(optional_config_path).await?;
            }
            Command::Jobs(command) => {
                command.exec(optional_config_path).await?;
            }
            Command::Modules(command) => {
                command.exec()?;
            }
            Command::Openapi(command) => {
                command.exec()?;
            }
            Command::Health(command) => {
                command.exec(optional_config_path).await?;
            }
            Command::Reload(command) => {
                command.exec()?;
            }
            Command::FixAcl(command) => {
                command.exec();
            }
            Command::Acl(command) => {
                command.exec();
            }
            Command::Dev(command) => {
                command.exec(optional_config_path).await?;
            }
        }
        Ok(())
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::cli::{health::HealthArgs, metrics::MetricsArgs};

mod health;
mod metrics;

#[derive(Parser, Debug, Clone)]
#[clap(name = "opentalk-roomserver")]
#[command(about)]
pub struct Args {
    /// Path of the configuration file.
    ///
    /// If present, exactly this config file will be used.
    ///
    /// If absent, the orchestrator looks for a config file in these locations and uses the first
    /// one that is found:
    ///
    /// - `roomserver.toml` in the current directory
    /// - `<XDG_CONFIG_HOME>/opentalk/roomserver.toml` (where `XDG_CONFIG_HOME` is usually
    ///   `~/.config`)
    /// - `/etc/opentalk/roomserver.toml`
    #[clap(short, long, help = "Specify path to configuration file")]
    pub(crate) config: Option<PathBuf>,

    #[clap(subcommand)]
    pub(crate) cmd: Option<SubCommand>,
}

/// The subcommands require
#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum SubCommand {
    /// Return the readiness state
    Health(HealthArgs),

    /// The metrics of the orchestrator
    Metrics(MetricsArgs),
}

pub async fn handle_subcommand(
    subcommand: SubCommand,
    config: Option<PathBuf>,
) -> anyhow::Result<()> {
    match subcommand {
        SubCommand::Health(args) => {
            health::health_check(config, args).await?;
        }
        SubCommand::Metrics(args) => metrics::fetch_metrics(config, args).await?,
    }

    Ok(())
}

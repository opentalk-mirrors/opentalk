// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use clap::{Parser, Subcommand};
use opentalk_version::InfoArgs;
use reqwest::Url;
mod license;

#[derive(Parser, Debug)]
pub struct Args {
    #[command(flatten)]
    pub info: InfoArgs,

    /// Path of the configuration file.
    ///
    /// If present, exactly this config file will be used.
    ///
    /// If absent, `recorder` looks for a config file in these locations and uses the first one that is found:
    ///
    /// - `config.toml` in the current directory (deprecated, for backwards compatibility only)
    /// - `recorder.toml` in the current directory
    /// - `<XDG_CONFIG_HOME>/opentalk/recorder.toml` (where `XDG_CONFIG_HOME` is usually `~/.config`)
    /// - `/etc/opentalk/recorder.toml`
    #[clap(short, long, verbatim_doc_comment)]
    pub config: Option<String>,

    #[clap(subcommand)]
    pub(crate) command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Return the readiness state
    Health {
        /// The monitoring endpoint can be provided optionally
        endpoint: Option<Url>,
    },
}

opentalk_version::build_info!();

pub fn print_info(info_args: &InfoArgs) {
    println!(
        "{}",
        build_info::BuildInfo::with_license(license::LICENSE.to_owned())
            .format(info_args)
            .unwrap_or("No Build information available.".to_string())
    );
}

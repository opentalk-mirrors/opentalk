// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod command;

use std::path::PathBuf;

use build_info::BuildInfo;
use clap::Parser;
use command::Command;
use opentalk_controller_core::Controller;
use opentalk_version::InfoArgs;

use crate::Result;

opentalk_version::build_info!();

#[derive(Parser, Debug, Clone)]
#[clap(name = "opentalk-controller")]
pub struct Args {
    /// Path of the configuration file.
    ///
    /// If present, exactly this config file will be used.
    ///
    /// If absent, `controller` looks for a config file in these locations and uses the first one that is found:
    ///
    /// - `config.toml` in the current directory (deprecated, for backwards compatiblity only)
    /// - `controller.toml` in the current directory
    /// - `<XDG_CONFIG_HOME>/opentalk/controller.toml` (where `XDG_CONFIG_HOME` is usually `~/.config`)
    /// - `/etc/opentalk/controller.toml`
    #[clap(short, long, verbatim_doc_comment)]
    pub config: Option<PathBuf>,

    /// Triggers a reload of reloadable configuration options (deprecated, use the `reload` subcommand instead)
    #[clap(long)]
    pub reload: bool,

    #[clap(subcommand)]
    cmd: Option<Command>,

    #[command(flatten)]
    pub(crate) info: InfoArgs,
}

impl Args {
    pub async fn exec(self) -> Result<()> {
        if self.info.should_print() {
            print_info(&self.info);
            return Ok(());
        }

        if self.reload {
            let current_exe = std::env::current_exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "opentalk-controller".to_string());
            println!(
                "The `--reload` argument is deprecated and will be removed in the future. Please execute `{current_exe} reload` instead."
            );
            command::reload::Command.exec()?;
            return Ok(());
        }

        if let Some(command) = self.cmd {
            command.exec(self.config.as_deref()).await?;
        } else {
            let controller = Controller::create(self.config).await?;
            controller.run().await?;
        }

        Ok(())
    }
}

pub(crate) fn print_info(info_args: &InfoArgs) {
    let build_info = BuildInfo::with_license(super::license::LICENSE.to_owned());
    if let Some(text) = build_info.format(info_args) {
        println!("{text}");
    }
}

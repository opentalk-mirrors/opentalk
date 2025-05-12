// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use build_info::BuildInfo;
use clap::Parser;
use opentalk_version::InfoArgs;

mod license;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[command(flatten)]
    info: InfoArgs,

    /// Path of the configuration file.
    ///
    /// If present, exactly this config file will be used.
    ///
    /// If absent, `obelisk` looks for a config file in these locations and uses the first one that is found:
    ///
    /// - `config.toml` in the current directory (deprecated, for backwards compatibility only)
    /// - `obelisk.toml` in the current directory
    /// - `<XDG_CONFIG_HOME>/opentalk/obelisk.toml` (where `XDG_CONFIG_HOME` is usually `~/.config`)
    /// - `/etc/opentalk/obelisk.toml`
    #[clap(short, long, verbatim_doc_comment)]
    pub config: Option<String>,
}

impl Args {
    /// Returns true if we want to startup the controller after we finished the cli part
    pub fn should_start(&self) -> bool {
        !self.info.should_print()
    }
}

/// Parses the CLI-Arguments into [`Args`]
///
/// Also runs (optional) cli commands if necessary
pub fn parse_args() -> Args {
    let args = Args::parse();
    if args.info.should_print() {
        print_info(&args.info);
    }
    args
}

opentalk_version::build_info!();

fn print_info(info_args: &InfoArgs) {
    let build_info = BuildInfo::with_license(license::LICENSE.to_owned());
    if let Some(text) = build_info.format(info_args) {
        println!("{text}");
    }
}

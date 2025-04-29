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
    #[clap(
        short,
        long,
        default_value = "config.toml",
        help = "Specify path to configuration file"
    )]
    pub config: String,
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

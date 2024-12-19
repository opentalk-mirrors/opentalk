// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use clap::{ArgAction, Parser};

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(short('V'), long, action=ArgAction::SetTrue, help = "Print version information")]
    version: bool,
}

impl Args {
    /// Returns true if we want to startup the controller after we finished the cli part
    pub fn should_start(&self) -> bool {
        !self.version
    }
}

/// Parses the CLI-Arguments into [`Args`]
///
/// Also runs (optional) cli commands if necessary
pub fn parse_args() -> Args {
    let args = Args::parse();
    if args.version {
        print_version();
    }
    args
}

opentalk_version::build_info!();

fn print_version() {
    println!("{}", build_info::BuildInfo::new());
}

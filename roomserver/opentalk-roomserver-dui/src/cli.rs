// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::path::PathBuf;

use clap::Parser;
use opentalk_version::InfoArgs;

opentalk_version::build_info!();

#[derive(Parser, Debug, Clone)]
#[command(about)]
pub(crate) struct Args {
    #[clap(short, long, help = "Specify path to configuration file")]
    pub(crate) config: Option<PathBuf>,

    #[command(flatten)]
    pub(crate) info: InfoArgs,
}

impl Args {
    /// Execute potential informational tasks like printing help messages or version info.
    ///
    /// When [`ProgramFlow::Exit`] is returned, the program should exit, otherwise the program
    /// should continue.
    pub(crate) fn run_tasks(&self) -> ProgramFlow {
        if !self.info.should_print() {
            return ProgramFlow::Continue;
        }
        let build_info = build_info::BuildInfo::with_license(LICENSE.to_owned());
        if let Some(text) = build_info.format(&self.info) {
            println!("{text}");
        }
        ProgramFlow::Exit
    }
}

/// Whether the program should start or exit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProgramFlow {
    /// Exit the program.
    Exit,

    /// The program should continue execution.
    Continue,
}

impl ProgramFlow {
    /// Returns `true` if the program flow is [`Exit`].
    ///
    /// [`Exit`]: ProgramFlow::Exit
    #[must_use]
    pub(crate) fn should_exit(self) -> bool {
        matches!(self, Self::Exit)
    }
}

const LICENSE: &str = "
Copyright 2025 OpenTalk GmbH

Licensed under the EUPL, Version 1.2 or – as soon they
will be approved by the European Commission - subsequent
versions of the EUPL (the \"Licence\");
You may not use this work except in compliance with the
Licence.
You may obtain a copy of the Licence at:

https://joinup.ec.europa.eu/software/page/eupl

Unless required by applicable law or agreed to in
writing, software distributed under the Licence is
distributed on an \"AS IS\" basis,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
express or implied.
See the Licence for the specific language governing
permissions and limitations under the Licence.

The source code is available at:

https://gitlab.opencode.de/opentalk/roomserver";

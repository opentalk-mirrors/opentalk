// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use clap::Subcommand;
use opentalk_roomserver_modules::{ListPrinter, MarkdownPrinter};

use crate::Result;

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Command {
    /// List available modules and their features
    List,

    /// Print a documentation of the modules (in Markdown)
    PrintDocumentation,
}

impl Command {
    pub fn exec(self) -> Result<()> {
        let registry = opentalk_roomserver_modules::setup_registry();

        match self {
            Self::List => {
                registry.print(&mut ListPrinter);
            }
            Self::PrintDocumentation => {
                registry.print(&mut MarkdownPrinter);
            }
        }
        Ok(())
    }
}

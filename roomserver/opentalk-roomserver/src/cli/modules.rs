// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use clap::Subcommand;
use opentalk_roomserver_modules::{ListPrinter, MarkdownPrinter, setup_registry};

#[derive(Subcommand, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Command {
    /// List available modules and their features
    List,

    /// Print a documentation of the modules (in Markdown)
    PrintDocumentation,
}

pub(crate) fn handle_command(command: Command) {
    let registry = setup_registry();

    match command {
        Command::List => {
            registry.print(&mut ListPrinter);
        }
        Command::PrintDocumentation => {
            registry.print(&mut MarkdownPrinter);
        }
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use clap::Parser;

#[derive(Debug, Clone, Parser)]
pub struct Command {
    _arbitrary_positional_arguments: Vec<String>,
}

impl Command {
    pub fn exec(self) {
        println!(
            "The API permission system has been improved, so the `fix-acl` subcommand is no longer necessary."
        );
    }
}

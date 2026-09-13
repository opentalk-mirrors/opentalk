// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Migrates the database schema

use std::path::Path;

use clap::Parser;
use opentalk_controller_core::load_settings_provider;
use snafu::ResultExt as _;

use crate::Result;

#[derive(Debug, Clone, Parser)]
pub struct Command {}

impl Command {
    pub async fn exec(self, optional_config_path: Option<&Path>) -> Result<()> {
        let settings = load_settings_provider(optional_config_path)?.get();
        let result = opentalk_db_storage::migrations::migrate_from_url(&settings.database.url)
            .await
            .whatever_context("Failed to migrate database")?;
        println!("{result:?}");
        Ok(())
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::settings_file;

pub const DEFAULT_DATABASE_MAX_CONNECTIONS: u32 = 100;

/// The runtime configuration for the database connection used by the OpenTalk controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Database {
    /// The url of the database service (in `postgres://…` schema).
    pub url: String,

    /// The maximum number of connections to allocate in the connection pool.
    pub max_connections: u32,
}

impl From<settings_file::Database> for Database {
    fn from(
        settings_file::Database {
            url,
            max_connections,
        }: settings_file::Database,
    ) -> Self {
        Self {
            url,
            max_connections: max_connections.unwrap_or(DEFAULT_DATABASE_MAX_CONNECTIONS),
        }
    }
}

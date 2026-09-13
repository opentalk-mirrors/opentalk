// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_database::DbConnection;
use opentalk_inventory::Inventory;

/// The database connection retrieved from the [`crate::DatabaseConnectionPool`].
#[derive(Debug)]
pub struct DatabaseConnection {
    pub(crate) inner: DbConnection,
}

impl Inventory for DatabaseConnection {}

impl DatabaseConnection {
    /// Create a new database connection wrapping a [`DbConnection`].
    pub fn new(connection: DbConnection) -> Self {
        Self { inner: connection }
    }

    /// Destructure the [`DatabaseConnection`], returning the wrapped [`DbConnection`].
    pub fn into_inner(self) -> DbConnection {
        self.inner
    }
}

impl AsRef<DbConnection> for DatabaseConnection {
    fn as_ref(&self) -> &DbConnection {
        &self.inner
    }
}

impl AsMut<DbConnection> for DatabaseConnection {
    fn as_mut(&mut self) -> &mut DbConnection {
        &mut self.inner
    }
}

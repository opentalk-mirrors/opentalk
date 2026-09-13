// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use opentalk_database::Db;
use opentalk_inventory::{AuthorizationInventory, Inventory, InventoryProvider};
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result, error::DatabaseSnafu};

/// The database connection pool.
#[derive(Debug)]
pub struct DatabaseConnectionPool {
    db: Arc<Db>,
}

#[async_trait::async_trait]
impl InventoryProvider for DatabaseConnectionPool {
    #[tracing::instrument(skip_all)]
    async fn get_inventory(&self) -> Result<Box<dyn Inventory>> {
        let connection = self.get_connection().await?;
        Ok(Box::new(connection))
    }

    #[tracing::instrument(skip_all)]
    async fn get_authorization_inventory(&self) -> Result<Box<dyn AuthorizationInventory>> {
        let inventory = self.get_inventory().await?;
        Ok(inventory)
    }
}

impl DatabaseConnectionPool {
    /// Create a new database connection pool wrapping a [`Db`].
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    /// Get a connection from the database pool.
    pub async fn get_connection(&self) -> Result<DatabaseConnection> {
        let inner = self.db.get_conn().await.context(DatabaseSnafu)?;
        Ok(DatabaseConnection { inner })
    }
}

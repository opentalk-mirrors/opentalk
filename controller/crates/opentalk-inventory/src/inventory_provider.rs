// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{AuthorizationInventory, Inventory, Result};

/// A provider for connections to the inventory backend. Could for example be a
/// database connection pool.
#[cfg_attr(feature = "mockall", mockall::automock)]
#[async_trait::async_trait]
pub trait InventoryProvider: Sync + Send + std::fmt::Debug {
    /// Get an instance of the inventory access handle from the provider.
    ///
    /// For inventory providers that are implemented as a database connection
    /// pool, this is typically an available database connection from the pool.
    async fn get_inventory(&self) -> Result<Box<dyn Inventory>>;

    /// Get an instance of the authorization inventory access handle from the provider.
    ///
    /// See [`Self::get_inventory`] for further information.
    async fn get_authorization_inventory(&self) -> Result<Box<dyn AuthorizationInventory>>;
}

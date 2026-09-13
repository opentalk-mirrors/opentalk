// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_inventory as inventory;
use snafu::Snafu;

/// Error performing the authorization.
#[derive(Debug, Snafu)]
pub enum AuthorizationError {
    /// Synchronization between instances failed
    SynchronizationFailed,

    /// Connecting to inventory failed
    #[snafu(display("Error connecting to inventory: {source}"))]
    InventoryConnection {
        /// Source inventory error
        source: inventory::Error,
    },
}

impl From<inventory::Error> for AuthorizationError {
    fn from(err: inventory::Error) -> Self {
        Self::InventoryConnection { source: err }
    }
}

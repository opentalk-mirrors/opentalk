// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::Result;

/// A trait for managing transactions in an inventory.
#[async_trait::async_trait]
pub trait TransactionManager {
    /// Begin a new transaction.
    async fn begin_transaction(&mut self) -> Result<()>;

    /// Rollback a currently active transaction.
    async fn rollback_transaction(&mut self) -> Result<()>;

    /// Commit a currently active transaction.
    async fn commit_transaction(&mut self) -> Result<()>;
}

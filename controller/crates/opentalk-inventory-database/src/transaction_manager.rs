// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel_async::{AnsiTransactionManager, TransactionManager as _};
use opentalk_inventory::{Error, TransactionManager};

use crate::{DatabaseConnection, Result};

#[async_trait::async_trait]
impl TransactionManager for DatabaseConnection {
    async fn begin_transaction(&mut self) -> Result<()> {
        AnsiTransactionManager::begin_transaction(&mut self.inner)
            .await
            .map_err(|_| Error::BrokenTransactionManager)
    }

    async fn rollback_transaction(&mut self) -> Result<()> {
        AnsiTransactionManager::rollback_transaction(&mut self.inner)
            .await
            .map_err(|_| Error::BrokenTransactionManager)
    }

    async fn commit_transaction(&mut self) -> Result<()> {
        AnsiTransactionManager::commit_transaction(&mut self.inner)
            .await
            .map_err(|_| Error::BrokenTransactionManager)
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_database::DatabaseError;
use opentalk_db_storage::queries::module_resources::types::JsonOperationError;
use opentalk_inventory_common::error::InventoryBackendError;
use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum Error {
    /// An error happened in the database backend.
    Database { source: DatabaseError },

    /// An error happened during a JSON operation.
    JsonOperation { source: JsonOperationError },
}

impl Error {
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            Error::Database {
                source: DatabaseError::NotFound,
                ..
            }
        )
    }

    pub fn is_check_violation(&self) -> bool {
        matches!(
            self,
            Error::Database {
                source,
                ..
            } if source.is_check_violation()
        )
    }

    pub fn is_unique_violation(&self) -> bool {
        matches!(self, Error::Database { source, .. } if source.is_unique_violation())
    }
}

impl From<Error> for InventoryBackendError {
    fn from(e: Error) -> Self {
        let e: Box<dyn std::error::Error + Send + Sync> = Box::new(e);
        e.into()
    }
}

impl From<Error> for opentalk_inventory::Error {
    fn from(e: Error) -> Self {
        if e.is_not_found() {
            return Self::NotFound;
        }
        if e.is_check_violation() {
            return Self::ConstraintViolation;
        }
        if e.is_unique_violation() {
            return Self::UniqueViolation;
        }
        Self::InventoryBackend {
            source: InventoryBackendError::from(e),
        }
    }
}

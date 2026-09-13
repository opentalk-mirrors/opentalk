// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! OpenTalk Database connector, interface and connection handling

use diesel_async::{
    AsyncPgConnection,
    pooled_connection::deadpool::{BuildError, Object, PoolError},
};
use snafu::Snafu;

mod db;
mod metrics;
pub mod query_helper;

pub use db::Db;
pub use metrics::DatabaseMetrics;

/// Pooled connection alias
pub type DbConnection = metrics::MetricsConnection<Object<AsyncPgConnection>>;

/// Result type using [`DatabaseError`] as a default Error
pub type Result<T, E = DatabaseError> = std::result::Result<T, E>;

/// Error types for the database abstraction
#[derive(Debug, Snafu)]
pub enum DatabaseError {
    #[snafu(display("Database Error: `{message}`",))]
    Custom { message: String },

    #[snafu(display("Diesel Error: `{source}`",))]
    DieselError { source: diesel::result::Error },

    #[snafu(display("A requested resource could not be found"))]
    NotFound,

    #[snafu(display("Deadpool build error: `{source}`",), context(false))]
    DeadpoolBuildError { source: BuildError },

    #[snafu(display("Deadpool error: `{source}`",))]
    DeadpoolError { source: PoolError },

    #[snafu(context(false))]
    UrlParseError { source: url::ParseError },
}

impl DatabaseError {
    /// Returns `true` if the database error is [`NotFound`].
    ///
    /// [`NotFound`]: DatabaseError::NotFound
    #[must_use]
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound)
    }
    /// Returns `true` if a database check has been violated
    #[must_use]
    pub fn is_check_violation(&self) -> bool {
        matches!(
            self,
            Self::DieselError {
                source: diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::CheckViolation,
                    _
                )
            }
        )
    }
    /// Returns `true` if a database unique constraint has been violated
    #[must_use]
    pub fn is_unique_violation(&self) -> bool {
        matches!(
            self,
            Self::DieselError {
                source: diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _
                )
            }
        )
    }
}

impl From<diesel::result::Error> for DatabaseError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => Self::NotFound,
            source => Self::DieselError { source },
        }
    }
}

pub trait OptionalExt<T, E> {
    fn optional(self) -> Result<Option<T>, E>;
}

impl<T> OptionalExt<T, DatabaseError> for Result<T, DatabaseError> {
    fn optional(self) -> Result<Option<T>, DatabaseError> {
        match self {
            Ok(t) => Ok(Some(t)),
            Err(DatabaseError::NotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::DatabaseError;

    #[test]
    fn test_database_error_from_implementation() {
        // The `diesel::result::Error::NotFound` should also be a
        // `DatabaseError::NotFound` and never be a `DatabaseError::DieselError`
        // All other cases should be `DatabaseError::DieselError`
        assert!(matches!(
            Into::<DatabaseError>::into(diesel::result::Error::NotFound),
            DatabaseError::NotFound,
        ));
        assert!(!matches!(
            Into::<DatabaseError>::into(diesel::result::Error::NotFound),
            DatabaseError::DieselError {
                source: diesel::result::Error::NotFound
            },
        ));
        assert!(matches!(
            Into::<DatabaseError>::into(diesel::result::Error::NotInTransaction),
            DatabaseError::DieselError {
                source: diesel::result::Error::NotInTransaction
            },
        ));
    }
}

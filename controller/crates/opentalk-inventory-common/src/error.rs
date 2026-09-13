// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains the error type that is returned form the traits defined in the opentalk-inventory crate.
//!
//! [snafu::whatever] can be used with this type as well if necessary.

use std::fmt::Display;

use snafu::Snafu;

/// The error returned from function calls to the inventory facade provider.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    /// An error happened in the inventory backend.
    InventoryBackend {
        /// The cause of the error
        source: InventoryBackendError,
    },

    /// An error occurred when attempting to begin, rollback or finish a transaction.
    BrokenTransactionManager,

    /// The requested entity was not found.
    NotFound,

    /// A database constraint was violated.
    ConstraintViolation,

    /// A database unique constraint was violated.
    UniqueViolation,

    /// A custom error with just a message.
    #[snafu(whatever, display("{message}"))]
    Custom {
        /// The custom error message.
        message: String,

        /// An optional source error.
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>,Some)))]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl Error {
    /// Returns `true` if the error is [`NotFound`].
    ///
    /// [`NotFound`]: Error::NotFound
    #[must_use]
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound)
    }

    /// Returns `true` if the error is [`UniqueViolation`].
    ///
    /// [`UniqueViolation`]: Error::UniqueViolation
    #[must_use]
    pub fn is_unique_violation(&self) -> bool {
        matches!(self, Self::UniqueViolation)
    }

    /// Returns `true` if the error is [`ConstraintViolation`].
    ///
    /// [`ConstraintViolation`]: Error::ConstraintViolation
    #[must_use]
    pub fn is_constraint_violation(&self) -> bool {
        matches!(self, Self::ConstraintViolation)
    }
}

/// An error that can be returned from the inventory backend.
#[derive(Debug)]
pub struct InventoryBackendError(Box<dyn std::error::Error + Send + Sync>);

impl From<Box<dyn std::error::Error + Send + Sync>> for InventoryBackendError {
    fn from(value: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self(value)
    }
}

impl std::error::Error for InventoryBackendError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}
impl Display for InventoryBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<InventoryBackendError> for Error {
    fn from(source: InventoryBackendError) -> Self {
        Error::InventoryBackend { source }
    }
}

// SPDX-License-Identifier: EUPL-1.2
//
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{convert::Infallible, fmt::Debug};

/// Marker trait to allow us to convert the `SignalingModule::Error` into a [`SignalingModuleError`]
pub trait ModuleError: Debug + Send {}

impl ModuleError for Infallible {}

/// The error type returned by signaling module event handlers
#[derive(Debug)]
pub enum SignalingModuleError<E> {
    /// An non-fatal internal error occurred
    Internal(anyhow::Error),
    /// A fatal error occurred.
    ///
    /// This is considered to be unrecoverable, the module will be flagged as broken and deactivated
    Fatal(FatalError),
    /// The module specific error
    ///
    /// Is turned into a websocket message and returned to the command issuing participant
    Module(E),
    /// The requested operation is not supported in the current context
    NotSupported,
}

impl<E> From<anyhow::Error> for SignalingModuleError<E> {
    fn from(err: anyhow::Error) -> Self {
        Self::Internal(err)
    }
}

impl<E: ModuleError> From<E> for SignalingModuleError<E> {
    fn from(err: E) -> Self {
        Self::Module(err)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("An unrecoverable error occurred")]
pub struct FatalError(#[source] pub anyhow::Error);

impl<E> From<FatalError> for SignalingModuleError<E> {
    fn from(err: FatalError) -> Self {
        Self::Fatal(err)
    }
}

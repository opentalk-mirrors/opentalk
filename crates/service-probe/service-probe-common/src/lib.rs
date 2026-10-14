// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Service probe State type
//!
//! This crate provides the definition of the `ServiceState` type.
#![deny(
    bad_style,
    missing_debug_implementations,
    missing_docs,
    overflowing_literals,
    patterns_in_fns_without_body,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

/// The state of a service
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ServiceState {
    /// The service is starting up.
    Up,

    /// The service is started and ready to process requests.
    Ready,
}

impl ServiceState {
    /// Get the [`str`] representation of the [`ServiceState`].
    pub const fn as_str(&self) -> &'static str {
        match self {
            ServiceState::Up => "UP",
            ServiceState::Ready => "READY",
        }
    }
}

impl std::fmt::Display for ServiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

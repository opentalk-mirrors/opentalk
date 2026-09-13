// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Common types used in the OpenTalk data inventory facade crate.

pub mod error;

/// The result type typically used for functions in this crate.
pub type Result<T, E = error::Error> = std::result::Result<T, E>;

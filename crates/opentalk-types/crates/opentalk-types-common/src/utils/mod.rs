// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Helpful utilities used in this crate, but also useful outside of it.

#[cfg(feature = "serde")]
pub mod comma_separated;
#[cfg(feature = "serde")]
pub mod duration_seconds;
#[cfg(feature = "serde")]
pub mod duration_seconds_option;

mod example_data;

pub use example_data::ExampleData;

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Test utility functions for use with the module tester and the controller
pub use ::serde_json;
#[cfg(feature = "controller")]
pub use common::{ROOM_ID, TestContext, TestUser, USER_1, USER_2, USERS};
pub use pretty_assertions::assert_eq;

#[cfg(feature = "controller")]
pub mod common;

#[cfg(feature = "controller")]
pub mod redis;

#[cfg(feature = "database")]
pub mod database;

#[cfg(feature = "object-storage")]
pub mod object_storage;

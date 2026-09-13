// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Data types related to the roomserver

mod device_secret;
mod token;

pub use device_secret::{DEVICE_SECRET_MAX_LENGTH, DEVICE_SECRET_MIN_LENGTH, DeviceSecret};
pub use token::Token;

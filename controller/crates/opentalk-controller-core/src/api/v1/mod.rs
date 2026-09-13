// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! OpenTalk controller REST API v1 service handlers

pub use response::{ApiResponse, DefaultApiResult};

pub mod events;
pub mod middleware;
pub mod response;

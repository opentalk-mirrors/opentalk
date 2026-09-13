// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Modules for external HTTP APIs
//!
//! Versions REST APIs are in v{VERSION}
//! APIs for use with our own frontend lie in internal
//! These directory map to the path prefix `/internal` or `/v1`

pub mod headers;
pub mod internal;
pub mod livekit;
pub mod signaling;
pub mod upload;
pub mod v1;

pub use opentalk_controller_api_actix_web::utoipa::responses;

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Middleware for enforcing authorization rules to OpenTalk Controller API endpoint access.

mod authorization_service;
mod authorization_transform;

pub use authorization_service::AuthorizationService;
pub use authorization_transform::AuthorizationTransform;

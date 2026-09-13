// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Authorization middleware for the OpenTalk Controller WebAPI endpoints.

pub mod authorization;

#[cfg(feature = "actix-web")]
pub mod middleware;

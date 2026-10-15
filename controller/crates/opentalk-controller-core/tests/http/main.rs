// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Black-box tests of the controller HTTP API.
//!
//! The tests send real HTTP requests to the application listening on a local port. They describe
//! the externally observable behavior, including oddities that clients may rely on, so that the
//! same tests can verify the behavior after the web framework is exchanged.

mod authentication;
mod harness;
mod json_errors;
mod routing;

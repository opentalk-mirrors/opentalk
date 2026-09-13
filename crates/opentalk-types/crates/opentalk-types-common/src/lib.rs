// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Common data types for OpenTalk.
//!
//! This crate contains data types that are commonly used in the OpenTalk !
//! APIs.

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

pub mod api_well_known;
pub mod assets;
pub mod auth;
pub mod call_in;
pub mod collections;
pub mod email;
pub mod events;
pub mod features;
pub mod module_resources;
pub mod modules;
pub mod order;
pub mod pagination;
pub mod rooms;
pub mod roomserver;
pub mod shared_folders;
pub mod streaming;
pub mod tariffs;
pub mod tenants;
pub mod time;
pub mod training_participation_report;
pub mod users;
pub mod utils;

mod macros;

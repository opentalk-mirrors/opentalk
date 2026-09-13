// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Common identifier types for OpenTalk crates
//!
//! These types are usually re-exported in `opentalk-types-common`, so this
//! crate is useless as a direct dependency in most cases, instead the
//! re-exported types should be used.

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

mod identifier;

use identifier::Identifier;

pub mod asset_file_kind;
pub mod feature_id;
pub mod module_id;

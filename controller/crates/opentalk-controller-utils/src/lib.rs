// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Utilities used in the OpenTalk controller

#![warn(
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

mod capture_api_error;
mod module_features;

pub mod deletion;
pub mod event;
pub mod opencloud;

pub use capture_api_error::CaptureApiError;
pub use module_features::{
    FeatureRequiredError, TariffResourceExt, get_tariff_for_room, get_tariff_for_user,
};

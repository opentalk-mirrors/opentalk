// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Test helpers exposed behind the `test-util` feature flag.
//!
//! Consumers (including this crate's own tests and downstream crates such as
//! `opentalk-test-util` and `opentalk-controller-api-authorization-database`)
//! enable the `test-util` feature on `opentalk-controller-settings` to pull
//! these helpers in. They are intentionally not part of the public API at
//! build time without the flag.

use crate::SettingsProvider;

/// Construct a [`SettingsProvider`] from the bundled minimal example raw
/// settings.
///
/// Intended for tests that need a ready-to-use [`SettingsProvider`] without
/// touching the filesystem.
pub fn settings_provider_from_example_raw_settings() -> SettingsProvider {
    let raw_settings = crate::settings_file::settings_raw_minimal_example();
    SettingsProvider::new_raw(raw_settings).unwrap()
}

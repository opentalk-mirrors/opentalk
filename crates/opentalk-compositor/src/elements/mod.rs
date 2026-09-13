// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;

pub(crate) mod matroskas3sink;

/// # Errors
///
/// Returns an error if Gstreamer is not initialized or this function was already called in this proccess.
pub fn register_all() -> Result<()> {
    matroskas3sink::register()
}

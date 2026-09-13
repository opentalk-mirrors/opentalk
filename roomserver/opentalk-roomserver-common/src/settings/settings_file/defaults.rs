// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

/// Some settings that apply for the whole installation.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Defaults {
    /// Flag indicating whether the screen share requires an explicit permission.
    pub screen_share_requires_permission: bool,
}

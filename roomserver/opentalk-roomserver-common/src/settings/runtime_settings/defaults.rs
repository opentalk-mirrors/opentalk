// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::settings::settings_file;

/// Some settings that apply for the whole installation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Defaults {
    /// Flag indicating whether the screen share requires an explicit permission.
    pub screen_share_requires_permission: bool,
}

impl From<settings_file::defaults::Defaults> for Defaults {
    fn from(value: settings_file::defaults::Defaults) -> Self {
        Self {
            screen_share_requires_permission: value.screen_share_requires_permission,
        }
    }
}

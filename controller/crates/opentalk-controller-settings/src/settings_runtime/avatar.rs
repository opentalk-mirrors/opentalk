// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::settings_file;

/// The default libravatar url.
pub const DEFAULT_LIBRAVATAR_URL: &str = "https://seccdn.libravatar.org/avatar/";

/// Avatar settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Avatar {
    /// The libravatar URL.
    pub libravatar_url: String,
}

impl Default for Avatar {
    fn default() -> Self {
        Self {
            libravatar_url: DEFAULT_LIBRAVATAR_URL.to_string(),
        }
    }
}

impl From<settings_file::Avatar> for Avatar {
    fn from(settings_file::Avatar { libravatar_url }: settings_file::Avatar) -> Self {
        Self {
            libravatar_url: libravatar_url.unwrap_or_else(|| DEFAULT_LIBRAVATAR_URL.to_string()),
        }
    }
}

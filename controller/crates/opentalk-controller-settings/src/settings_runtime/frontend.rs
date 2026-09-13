// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use url::Url;

use crate::settings_file;

/// Frontend settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frontend {
    /// The base URL of the frontend.
    pub base_url: Url,
}

impl From<settings_file::Frontend> for Frontend {
    fn from(settings_file::Frontend { base_url }: settings_file::Frontend) -> Self {
        Self { base_url }
    }
}

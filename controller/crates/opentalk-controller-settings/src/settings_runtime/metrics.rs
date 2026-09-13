// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::settings_file;

/// Metrics settings.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Metrics {
    /// The list of allowed clients.
    pub allowlist: Vec<cidr::IpInet>,
}

impl From<settings_file::Metrics> for Metrics {
    fn from(settings_file::Metrics { allowlist }: settings_file::Metrics) -> Self {
        Self { allowlist }
    }
}

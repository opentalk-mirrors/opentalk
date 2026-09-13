// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::settings_file;

/// Etcd settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Etcd {
    /// The list of URLs where the etcd service is reachable.
    pub urls: Vec<url::Url>,
}

impl From<settings_file::Etcd> for Etcd {
    fn from(settings_file::Etcd { urls }: settings_file::Etcd) -> Self {
        Self { urls }
    }
}

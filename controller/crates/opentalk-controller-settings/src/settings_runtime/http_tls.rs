// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::path::PathBuf;

use crate::settings_file;

/// TLS configuration for the HTTP service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpTls {
    /// The path to the certificate file.
    pub certificate: PathBuf,

    /// The path to the private key file.
    pub private_key: PathBuf,
}

impl From<settings_file::HttpTls> for HttpTls {
    fn from(
        settings_file::HttpTls {
            certificate,
            private_key,
        }: settings_file::HttpTls,
    ) -> Self {
        HttpTls {
            certificate,
            private_key,
        }
    }
}

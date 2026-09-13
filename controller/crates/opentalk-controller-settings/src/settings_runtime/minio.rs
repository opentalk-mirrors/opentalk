// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::settings_file;

/// MinIO settings.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct MinIO {
    /// The URI of the S3 storage.
    pub uri: String,

    /// The bucket in the S3 storage.
    pub bucket: String,

    /// AWS region defaults to `unknown`
    pub region: Option<String>,

    /// Force path style defaults to false
    pub force_path_style: Option<bool>,

    /// The access key to the storage.
    pub access_key: String,

    /// The secret key to the storage.
    pub secret_key: String,
}

impl From<settings_file::MinIO> for MinIO {
    fn from(
        settings_file::MinIO {
            uri,
            bucket,
            access_key,
            secret_key,
            region,
            force_path_style,
        }: settings_file::MinIO,
    ) -> Self {
        Self {
            uri,
            bucket,
            access_key,
            secret_key,
            region,
            force_path_style,
        }
    }
}

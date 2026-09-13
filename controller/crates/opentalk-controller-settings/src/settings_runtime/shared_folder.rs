// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::settings_file;

/// `SharedFolder` settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SharedFolder {
    /// `SharedFolder` on a Nextcloud instance.
    Nextcloud {
        /// The URL of the Nextcloud instance.
        url: url::Url,

        /// The username used for logging in.
        username: String,

        /// The password used for logging in.
        password: String,

        /// The directory inside which the shared folder is created.
        directory: String,

        /// The expiry duration of the share.
        expiry: Option<u64>,
    },

    /// `SharedFolder` on a `OpenCloud` instance.
    Opencloud {
        /// The URL of the `OpenCloud` instance.
        url: url::Url,

        /// The username used for logging in.
        username: String,

        /// The password used for logging in.
        password: String,

        /// The directory inside which the shared folder is created.
        directory: String,

        /// The expiry duration of the share.
        expiry: Option<u64>,
    },
}

impl From<settings_file::SharedFolder> for SharedFolder {
    fn from(value: settings_file::SharedFolder) -> Self {
        match value {
            settings_file::SharedFolder::Nextcloud {
                url,
                username,
                password,
                directory,
                expiry,
            } => Self::Nextcloud {
                url,
                username,
                password,
                directory,
                expiry,
            },
            settings_file::SharedFolder::Opencloud {
                url,
                username,
                password,
                directory,
                expiry,
            } => Self::Opencloud {
                url,
                username,
                password,
                directory,
                expiry,
            },
        }
    }
}

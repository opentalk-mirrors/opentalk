// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::settings_file;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redis {
    pub url: url::Url,
}

impl From<settings_file::RedisConfig> for Redis {
    fn from(settings_file::RedisConfig { url }: settings_file::RedisConfig) -> Self {
        Self {
            url: url.unwrap_or_else(default_url),
        }
    }
}

fn default_url() -> url::Url {
    url::Url::try_from("redis://localhost:6379/").expect("Invalid default redis URL")
}

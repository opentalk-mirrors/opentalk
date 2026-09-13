// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RedisConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
}

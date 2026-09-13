// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Database {
    pub url: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_connections: Option<u32>,
}

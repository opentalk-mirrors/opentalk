// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

use crate::common::HttpCorsAllowedOrigins;

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct HttpCors {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_origin: Option<HttpCorsAllowedOrigins>,
}

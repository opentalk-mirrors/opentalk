// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_service_auth::service::ApiKeys;
use serde::Deserialize;

use super::{HttpCors, HttpTls};

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Http {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub addr: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls: Option<HttpTls>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cors: Option<HttpCors>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_api_keys: Option<ApiKeys>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upload_size_limit: Option<usize>,
}

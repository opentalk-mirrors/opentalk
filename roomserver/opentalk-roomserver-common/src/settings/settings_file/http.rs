// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_service_auth::service::ApiKeys;
use serde::Deserialize;
use url::Url;

/// Settings for the HTTP server
#[derive(Debug, Clone, Deserialize)]
pub struct Http {
    /// The IP address that the HTTP server should bind to
    #[serde(default)]
    pub address: Option<String>,

    /// The port that the HTTP server should use
    #[serde(default = "default_port")]
    pub port: u16,

    /// The URL that is reachable by internal services
    pub service_url: Option<Url>,

    /// The publicly reachable URL of this server
    pub public_url: Url,

    /// The configured API token for service endpoints
    pub api_keys: ApiKeys,

    // Enable the OpenAPI endpoint under `/v1/openapi.json` and the corresponding
    // swagger endpoint under `/swagger`.
    #[serde(default)]
    pub enable_openapi: bool,
}

const fn default_port() -> u16 {
    11333
}

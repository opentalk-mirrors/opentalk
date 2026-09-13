// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::net::{Ipv4Addr, Ipv6Addr, TcpListener};

use opentalk_service_auth::service::ApiKeys;

use crate::settings::settings_file;

/// Settings for the HTTP server
#[derive(Debug, Clone)]
pub struct Http {
    /// The IP address that the HTTP server should bind to
    pub address: String,

    /// The port that the HTTP server should use
    pub port: u16,

    /// The URL that is reachable by internal services
    pub service_url: Option<url::Url>,

    /// The publicly reachable URL of this server
    pub public_url: url::Url,

    /// The API keys for service endpoints
    pub api_keys: ApiKeys,

    // Enable the OpenAPI endpoint under `/v1/openapi.json` and the corresponding
    // swagger endpoint under `/swagger`.
    pub enable_openapi: bool,
}

impl From<settings_file::http::Http> for Http {
    fn from(value: settings_file::http::Http) -> Self {
        let address = match value.address {
            Some(address) => address,
            None => {
                if is_ipv6_available() {
                    Ipv6Addr::UNSPECIFIED.to_string()
                } else {
                    Ipv4Addr::UNSPECIFIED.to_string()
                }
            }
        };

        Self {
            address,
            port: value.port,
            service_url: value.service_url,
            public_url: value.public_url,
            api_keys: value.api_keys,
            enable_openapi: value.enable_openapi,
        }
    }
}

fn is_ipv6_available() -> bool {
    TcpListener::bind((Ipv6Addr::UNSPECIFIED, 0)).is_ok()
}

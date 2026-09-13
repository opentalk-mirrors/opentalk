// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::net::{IpAddr, Ipv4Addr};

use opentalk_service_auth::service::ApiKeys;
use serde::Deserialize;
use url::Url;

/// Settings for the HTTP server
#[derive(Debug, Clone, Deserialize)]
pub struct Http {
    /// The IP address that the HTTP server should bind to
    #[serde(default = "default_bind_address")]
    pub address: IpAddr,

    /// The port that the HTTP server should use
    #[serde(default = "default_port")]
    pub port: u16,

    /// The public url of the orchestrator. This must be reachable by clients because the
    /// orchestrator is the gateway for roomserver signaling.
    pub public_url: Url,

    /// The configured api keys for the orchestrators endpoints
    pub api_keys: ApiKeys,
}

pub(crate) const fn default_bind_address() -> IpAddr {
    IpAddr::V4(Ipv4Addr::UNSPECIFIED)
}

const fn default_port() -> u16 {
    11222
}

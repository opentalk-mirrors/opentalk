// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use cidr::IpInet;
use serde::Deserialize;
/// Configuration for the orchestrators `/metrics` endpoint
///
/// By default, the /metrics endpoint expects a bearer JWT using the orchestrators API key id and
/// secret
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Metrics {
    /// Disable all token based authorization for the `/metrics` endpoint
    ///
    /// The IP address allowlist still applies
    #[serde(default)]
    pub disable_token_auth: bool,

    /// Use a plain bearer token string instead of a JWT with the orchestrators API keys.
    ///
    /// The /metrics endpoint will expect a the `AUTHORIZATION` header that contains
    /// `Bearer <token>`
    #[serde(default)]
    pub bearer_token: Option<String>,

    /// Only allow contained IP addresses to access the metric endpoint
    ///
    /// An empty list will deny every request
    #[serde(default)]
    pub allowlist: Option<Vec<IpInet>>,
}

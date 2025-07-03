// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use openidconnect::{ClientId, ClientSecret, IssuerUrl};

#[derive(Debug)]
pub struct ClientConfig {
    pub auth: AuthConfig,
    pub controller: ControllerConfig,
}

#[derive(Debug)]
pub struct AuthConfig {
    pub issuer: IssuerUrl,
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
}

#[derive(Debug)]
pub struct ControllerConfig {
    pub domain: String,
    pub insecure: bool,
}

impl ControllerConfig {
    #[must_use]
    pub(crate) fn websocket_url(&self) -> String {
        let scheme = if self.insecure { "ws" } else { "wss" };

        format!("{scheme}://{}/signaling", self.domain)
    }

    #[must_use]
    pub(crate) fn v1_api_base_url(&self) -> String {
        let scheme = if self.insecure { "http" } else { "https" };

        format!("{scheme}://{}/v1", self.domain)
    }
}

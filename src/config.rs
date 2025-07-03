// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::authentication::Authentication;
use core::fmt;
use openidconnect::{
    ClientId, ClientSecret, IssuerUrl, ResourceOwnerPassword, ResourceOwnerUsername,
};

#[derive(Debug)]
pub struct ClientConfig {
    pub auth: AuthConfig,
    pub controller: ControllerConfig,
}

pub enum AuthConfig {
    /// Use OIDC client credentials to authenticate with a service account
    ClientCredentials(ClientCredentialsAuthConfig),

    /// Use the OIDC direct access credentials flow to authenticate
    DirectAccess(DirectionAccessCredentialsAuthConfig),

    /// No authentication, e.g. for guest users
    None,

    /// External implementation of the authentication mechanism
    Other(Box<dyn Authentication>),
}

impl fmt::Debug for AuthConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClientCredentials(arg0) => {
                f.debug_tuple("ClientCredentials").field(arg0).finish()
            }
            Self::DirectAccess(arg0) => f.debug_tuple("DirectAccess").field(arg0).finish(),
            Self::None => f.debug_tuple("None").finish(),
            Self::Other(..) => f
                .debug_tuple("Other")
                .field(&"Box<dyn Authentication>")
                .finish(),
        }
    }
}

#[derive(Debug)]
pub struct ClientCredentialsAuthConfig {
    pub issuer: IssuerUrl,
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
}

#[derive(Debug)]
pub struct DirectionAccessCredentialsAuthConfig {
    pub issuer: IssuerUrl,
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    pub username: ResourceOwnerUsername,
    pub password: ResourceOwnerPassword,
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

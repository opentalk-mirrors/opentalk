// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use core::fmt;
use std::str::FromStr;

use openidconnect::{
    ClientId, ClientSecret, IssuerUrl, ResourceOwnerPassword, ResourceOwnerUsername, url,
};
use opentalk_service_auth::ApiKey;
use reqwest::Url;
use serde::{Deserialize, Deserializer, de};

use crate::authentication::Authentication;

#[derive(Debug)]
pub struct ClientConfig {
    pub auth: AuthConfig,
    pub controller: BaseUrl,
}

pub enum AuthConfig {
    ApiKey(ApiKey),

    /// Use OIDC client credentials to authenticate with a service account
    ClientCredentials(ClientCredentialsAuthConfig),

    /// Use the OIDC direct access credentials flow to authenticate
    DirectAccess(DirectAccessCredentialsAuthConfig),

    /// No authentication, e.g. for guest users
    None,

    /// External implementation of the authentication mechanism
    Other(Box<dyn Authentication>),
}

impl fmt::Debug for AuthConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApiKey(_) => f.debug_tuple("ApiKey").finish(),
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
pub struct DirectAccessCredentialsAuthConfig {
    pub issuer: IssuerUrl,
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    pub username: ResourceOwnerUsername,
    pub password: ResourceOwnerPassword,
}

#[derive(Debug, thiserror::Error)]
pub enum BaseUrlError {
    #[error("Base url cannot have a query: '{0}'")]
    QueryNotSupported(String),
    #[error("Invalid path '{0}', the path for the base url must end with a trailing slash")]
    InvalidPath(String),
    #[error("Invalid scheme '{0}', scheme must be 'http' or 'https'")]
    InvalidScheme(String),
}

#[derive(Debug, thiserror::Error)]
pub enum BaseUrlFromStrError {
    #[error(transparent)]
    InvalidUrl(#[from] url::ParseError),
    #[error(transparent)]
    InvalidBaseUrl(#[from] BaseUrlError),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BaseUrl(Url);

impl BaseUrl {
    /// See [`Url::join`]
    pub fn join(&self, input: impl AsRef<str>) -> Result<Url, url::ParseError> {
        self.0.join(input.as_ref())
    }

    pub fn to_websocket_url(mut self) -> Url {
        match self.0.scheme() {
            "https" => self.0.set_scheme("wss").expect("wss is a new valid scheme"),
            "http" => self.0.set_scheme("ws").expect("wss is a new valid scheme"),
            _ => unreachable!("BaseUrl must contain http or https scheme url"),
        }

        self.0
    }
}

impl FromStr for BaseUrl {
    type Err = BaseUrlFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(s)?;
        let base_url = BaseUrl::try_from(url)?;
        Ok(base_url)
    }
}

impl TryFrom<Url> for BaseUrl {
    type Error = BaseUrlError;

    fn try_from(url: Url) -> Result<Self, Self::Error> {
        // Make sure the path is empty or ends with a /
        if !(url.path().is_empty() || url.path().ends_with('/')) {
            return Err(BaseUrlError::InvalidPath(url.path().into()));
        }

        if let Some(query) = url.query() {
            return Err(BaseUrlError::QueryNotSupported(query.into()));
        }

        match url.scheme() {
            "http" | "https" => {}
            other => return Err(BaseUrlError::InvalidScheme(other.into())),
        }

        Ok(BaseUrl(url))
    }
}

impl<'de> Deserialize<'de> for BaseUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let url = String::deserialize(deserializer)?;

        BaseUrl::from_str(&url).map_err(de::Error::custom)
    }
}

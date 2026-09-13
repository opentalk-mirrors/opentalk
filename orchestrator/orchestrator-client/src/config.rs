// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{fmt::Display, str::FromStr};

use opentalk_service_auth::ApiKey;
use serde::{Deserialize, Deserializer, de};
use url::{ParseError, Url};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum UrlError {
    #[error("Parsing error: {0}")]
    Parse(#[from] ParseError),
    #[error("Invalid scheme '{0}', scheme must be 'http' or 'https'")]
    InvalidScheme(String),
    #[error("Orchestrator url is not a base url: {0}")]
    NotABaseUrl(#[from] BaseUrlError),
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum BaseUrlError {
    #[error("Base url cannot have a query: '{0}'")]
    QueryNotSupported(String),
    #[error("Invalid path '{0}', the path for the base url must end with a trailing slash")]
    InvalidPath(String),
}

/// The configuration for the Orchestrator client
#[derive(Debug, Clone, Deserialize)]
pub struct OrchestratorConfig {
    /// The base url of the orchestrator
    pub url: OrchestratorBaseUrl,

    /// The key for the orchestrators API
    pub api_key: ApiKey,
}

/// The base URL of the Orchestrator
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestratorBaseUrl(Url);

impl OrchestratorBaseUrl {
    pub fn inner(&self) -> &Url {
        &self.0
    }

    pub fn into_inner(self) -> Url {
        self.0
    }

    /// Validates the scheme, path and query of the given [`Url`]
    fn validate(url: &Url) -> Result<(), UrlError> {
        match url.path() {
            "" | "/" => (),
            path => {
                if !path.ends_with('/') {
                    return Err(BaseUrlError::InvalidPath(path.into()).into());
                }
            }
        }

        if let Some(query) = url.query() {
            return Err(BaseUrlError::QueryNotSupported(query.into()).into());
        }

        match url.scheme() {
            "http" | "https" => Ok(()),
            other => Err(UrlError::InvalidScheme(other.into())),
        }
    }

    /// Build the url for the orchestrators register endpoint
    pub fn register_endpoint(&self) -> Result<Url, UrlError> {
        let mut url = self.0.join("register")?;

        match url.scheme() {
            "http" => {
                url.set_scheme("ws").expect("'ws' is valid scheme");
            }
            "https" => url.set_scheme("wss").expect("'wss' is valid scheme"),
            other => return Err(UrlError::InvalidScheme(other.into())),
        };

        Ok(url)
    }
}

impl<'de> Deserialize<'de> for OrchestratorBaseUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let url = String::deserialize(deserializer)?;

        OrchestratorBaseUrl::from_str(&url).map_err(de::Error::custom)
    }
}

impl std::fmt::Display for OrchestratorBaseUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl From<OrchestratorBaseUrl> for Url {
    fn from(val: OrchestratorBaseUrl) -> Self {
        val.0
    }
}

impl FromStr for OrchestratorBaseUrl {
    type Err = UrlError;

    fn from_str(url: &str) -> Result<Self, Self::Err> {
        let url: Url = Url::parse(url).map_err(UrlError::Parse)?;
        OrchestratorBaseUrl::validate(&url)?;

        Ok(Self(url))
    }
}

impl TryFrom<Url> for OrchestratorBaseUrl {
    type Error = UrlError;

    fn try_from(url: Url) -> Result<Self, Self::Error> {
        OrchestratorBaseUrl::validate(&url)?;

        Ok(Self(url))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use serde_json::json;

    use crate::{
        OrchestratorBaseUrl,
        config::{BaseUrlError, UrlError},
    };

    #[test]
    fn register_endpoint() {
        let url = OrchestratorBaseUrl::from_str("https://localhost").unwrap();
        let ws_url = url.register_endpoint().unwrap();

        assert_eq!(ws_url.as_str(), "wss://localhost/register")
    }

    #[test]
    fn register_endpoint_trailing_slash() {
        let url = OrchestratorBaseUrl::from_str("http://localhost/").unwrap();

        let ws_url = url.register_endpoint().unwrap();
        assert_eq!(ws_url.as_str(), "ws://localhost/register")
    }

    #[test]
    fn websocket_url_with_path() {
        let url = OrchestratorBaseUrl::from_str("https://localhost/proxy_path/").unwrap();

        let ws_url = url.register_endpoint().unwrap();
        assert_eq!(ws_url.as_str(), "wss://localhost/proxy_path/register")
    }

    #[test]
    fn invalid_scheme() {
        match OrchestratorBaseUrl::from_str("file://localhost/") {
            Ok(_) => panic!("expected error"),
            Err(e) => match e {
                UrlError::InvalidScheme(_) => (),
                _ => panic!("Expected different error"),
            },
        }
    }

    #[test]
    fn invalid_path() {
        match OrchestratorBaseUrl::from_str("https://localhost/proxy_path") {
            Ok(_) => panic!("expected error"),
            Err(e) => match e {
                UrlError::NotABaseUrl(BaseUrlError::InvalidPath(_)) => (),
                _ => panic!("Expected different error"),
            },
        }
    }

    #[test]
    fn with_query() {
        match OrchestratorBaseUrl::from_str("https://localhost/proxy_path?foo=bar") {
            Ok(_) => panic!("expected error"),
            Err(e) => match e {
                UrlError::NotABaseUrl(BaseUrlError::InvalidPath(_)) => (),
                _ => panic!("Expected different error"),
            },
        }
    }

    #[test]
    fn deserialize() {
        let json = json!("http://0.0.0.0:1234");

        let orchestrator_url: OrchestratorBaseUrl = serde_json::from_value(json).unwrap();

        assert_eq!(
            orchestrator_url,
            "http://0.0.0.0:1234"
                .parse::<OrchestratorBaseUrl>()
                .unwrap()
        )
    }

    #[test]
    fn deserialize_invalid_scheme() {
        let json = json!("file://0.0.0.0");

        if serde_json::from_value::<OrchestratorBaseUrl>(json).is_ok() {
            panic!("expected error")
        }
    }

    #[test]
    fn deserialize_invalid_path() {
        let json = json!("http://0.0.0.0:1234/hmmm.txt");

        if serde_json::from_value::<OrchestratorBaseUrl>(json).is_ok() {
            panic!("expected error")
        }
    }

    #[test]
    fn deserialize_query() {
        let json = json!("http://0.0.0.0:1234?foo=bar");

        if serde_json::from_value::<OrchestratorBaseUrl>(json).is_ok() {
            panic!("expected error")
        }
    }
}

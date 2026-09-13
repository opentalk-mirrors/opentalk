// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! Service related implementations

use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use jsonwebtoken::DecodingKey;
use opentalk_types_api_common::error::ApiError;
use serde::Deserialize;

use crate::{
    api_key::ApiKey,
    service::decoding_keys::{DecodingError, DecodingKeys},
};

#[cfg(feature = "actix")]
pub mod actix;
#[cfg(feature = "axum")]
pub mod axum;
mod decoding_keys;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum BuildMiddlewareError {
    #[error("No API keys configured")]
    NoKeysConfigured,
    #[error("Found duplicate API key identifier")]
    DuplicateKeyId,
}

/// A map of API keys and their identifiers
///
/// This type is usually built by deserializing a list of [`ApiKey`]s. Note that the
/// api keys can be deserialized as object or as string. E.g.:
///
/// ```toml
/// api_keys = [
///     # object representation:
///     { id = "roomserver", secret = "secret1" },
///     { id = "recorder", secret = "secret2" },
///     # string representation:
///     "controller:very_secret"
/// ]
/// ```
///
/// Because this type is deserialized as a list, some configurations have to be made to make this
/// type compatible with environment variables when using the `config` crate.
///
/// On the `Environment` struct the `try_parsing`, `list_separator` and `with_list_parse_key` fields
/// have to be set. The `with_list_parse_key` needs to contain the full path to the field of this
/// type.
///
/// ```rust
/// let config = Config::builder()
/// .add_source(
///     Environment::with_prefix("OT_TEST")
///         .prefix_separator("_")
///         .separator("__")
///         .try_parsing(true)
///         .list_separator(","),
///         .with_list_parse_key("http.api_keys")
/// )
/// .build()
/// .unwrap();
/// ```
///
/// The config above would allow an environment variable like this:
///
/// ```sh
/// OT_TEST_HTTP__API_KEYS="roomserver:secret1,recorder:secret2,controller:very_secret"
/// ```
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct ApiKeys(Vec<ApiKey>);

impl ApiKeys {
    pub fn new(keys: Vec<ApiKey>) -> Self {
        Self(keys)
    }

    pub fn inner(&self) -> &[ApiKey] {
        &self.0
    }

    pub fn into_inner(self) -> Vec<ApiKey> {
        self.0
    }

    /// Add an API key
    pub fn add_key(&mut self, key: ApiKey) {
        self.0.push(key);
    }

    /// Create a `axum`/`actix` middleware from the current map of API keys
    ///
    /// The middleware implementation depends on the enabled features.
    pub fn auth_middleware(&self) -> Result<ApiKeyAuthorization, BuildMiddlewareError> {
        if self.0.is_empty() {
            return Err(BuildMiddlewareError::NoKeysConfigured);
        }

        let mut decoding_keys = HashMap::new();

        for key in &self.0 {
            match decoding_keys.entry(key.id.clone()) {
                Entry::Occupied(_) => return Err(BuildMiddlewareError::DuplicateKeyId),
                Entry::Vacant(vacant_entry) => {
                    vacant_entry.insert(DecodingKey::from_secret(key.secret.0.as_bytes()));
                }
            }
        }

        Ok(ApiKeyAuthorization {
            decoding_keys: Arc::new(DecodingKeys::new(decoding_keys)),
        })
    }
}

/// The API key authorization middleware
///
/// Depending on the enabled features (`axum`/`actix`), this type implements the required traits to
/// be used as authorization middleware for the respective framework.
///
/// The middleware requires the `Bearer <JWT> format in the `authorization` header of incoming HTTP
/// requests. A request passes this middleware when the provided token is signed with one of the
/// configured API keys and the JWT key id (`kid`) configured key id.
#[derive(Debug, Clone)]
pub struct ApiKeyAuthorization {
    decoding_keys: Arc<DecodingKeys>,
}

/// The bearer prefix of the `AUTHORIZATION` header
const BEARER_PREFIX: &[u8] = b"bearer ";

/// Trims whitespaces and the 'Bearer' prefix from the given byte slice and returns the remaining
/// bytes
///
/// Returns `None` when the prefix isn't present or no bytes remain after the trim
fn trim_bearer_prefix(header: &[u8]) -> Option<&[u8]> {
    let (prefix, remaining) = header
        .trim_ascii()
        .split_first_chunk::<{ BEARER_PREFIX.len() }>()?;

    if !prefix.eq_ignore_ascii_case(BEARER_PREFIX) {
        return None;
    };

    if remaining.is_empty() {
        return None;
    }

    Some(remaining)
}

fn missing_token_error() -> ApiError {
    ApiError::unauthorized()
        .with_code("missing_api_token")
        .with_message("missing api token in authorization header")
}

fn invalid_token_error(error: DecodingError) -> ApiError {
    ApiError::unauthorized()
        .with_code("invalid_api_token")
        .with_message(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::env;

    use serde::Deserialize;

    use crate::{api_key::ApiKey, service::ApiKeys};

    #[test]
    fn add_key() {
        let mut api_keys = ApiKeys::default();

        let api_key = ApiKey::new("key_id", "secret");

        api_keys.add_key(api_key.clone());

        assert_eq!(api_keys.0.len(), 1);
        assert_eq!(api_keys.0[0], api_key);
    }

    #[derive(Deserialize)]
    struct TestConfig {
        http: HttpConfig,
    }

    #[derive(Deserialize)]
    struct HttpConfig {
        api_keys: ApiKeys,
        url: String,
    }

    #[test]
    fn deserialize_toml() {
        let toml = r#"        
        [http]
        url = ""
        api_keys= [
         { id = "roomserver", secret = "1234" },
         { id = "recorder", secret = "4321" },
         "controller:5678",
        ]
        "#;

        let TestConfig {
            http: HttpConfig { api_keys, .. },
        } = match toml::from_str(toml) {
            Ok(a) => a,
            Err(e) => panic!("{e}"),
        };

        assert_eq!(api_keys.0.len(), 3);
        assert_eq!(api_keys.0[0], ApiKey::new("roomserver", "1234"));
        assert_eq!(api_keys.0[1], ApiKey::new("recorder", "4321"));
        assert_eq!(api_keys.0[2], ApiKey::new("controller", "5678"));
    }

    #[test]
    fn test_config_crate() {
        use config::{Config, Environment};

        unsafe {
            env::set_var("OT_TEST_HTTP__API_KEYS", "roomserver:1234,recorder:4321");
            env::set_var("OT_TEST_HTTP__URL", "http://localhost");
        }

        let config = Config::builder()
            .add_source(
                Environment::with_prefix("OT_TEST")
                    .prefix_separator("_")
                    .separator("__")
                    .try_parsing(true)
                    .with_list_parse_key("http.api_keys")
                    .list_separator(","),
            )
            .build()
            .unwrap();

        let TestConfig {
            http: HttpConfig { api_keys, url },
        } = TestConfig::deserialize(config).unwrap();

        assert_eq!(api_keys.0.len(), 2);
        assert_eq!(api_keys.0[0], ApiKey::new("roomserver", "1234"));
        assert_eq!(api_keys.0[1], ApiKey::new("recorder", "4321"));
        assert_eq!(url, "http://localhost");
    }
}

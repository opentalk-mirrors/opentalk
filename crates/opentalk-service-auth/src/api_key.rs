// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! The api key for service authorization

use std::{str::FromStr, time::Duration};

use jsonwebtoken::{Algorithm, EncodingKey, Header, encode, get_current_timestamp};
use rand::{RngExt, distr::Alphanumeric, rng};
use serde::{Deserialize, Serialize};

use crate::{ApiKeyId, ApiKeySecret};

pub const JWT_EXPIRY: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, thiserror::Error)]
#[error("Failed to encode JSON web token: {0:?}")]
pub struct EncodingError(#[from] jsonwebtoken::errors::Error);

/// Claims for a minimal JSON Web Token
#[derive(Serialize)]
struct ExpiryClaims {
    /// Expiry
    exp: u64,
    /// Issued at
    iat: u64,
}

/// The API key
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ApiKey {
    /// The API key identifier
    pub id: ApiKeyId,
    /// The secret API key
    pub secret: ApiKeySecret,
}

impl ApiKey {
    /// Create a new API key
    pub fn new<I, K>(id: I, secret: K) -> Self
    where
        I: Into<ApiKeyId>,
        K: Into<ApiKeySecret>,
    {
        Self {
            id: id.into(),
            secret: secret.into(),
        }
    }

    /// Create a new JSON Web Token based on the API key
    ///
    /// The created token has a random nonce and default expiry of [`JWT_EXPIRY`]
    pub fn generate_jwt(&self) -> Result<String, EncodingError> {
        let iat = get_current_timestamp();
        let exp = iat + JWT_EXPIRY.as_secs();

        let mut header = Header::new(Algorithm::HS256);

        let nonce = rng()
            .sample_iter(Alphanumeric)
            .take(16)
            .map(|ascii_byte| ascii_byte as char)
            .collect();

        header.kid = Some(self.id.0.clone());
        header.nonce = Some(nonce);

        Ok(encode(
            &header,
            &ExpiryClaims { exp, iat },
            &EncodingKey::from_secret(self.secret.0.as_bytes()),
        )?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseApiKeyError {
    #[error("missing colon delimiter")]
    MissingColon,
    #[error("key id must not be empty")]
    EmptyKeyId,
    #[error("key secret must not be empty")]
    EmptyKeySecret,
}

impl FromStr for ApiKey {
    type Err = ParseApiKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(':') {
            Some((key_id, secret)) => {
                if key_id.is_empty() {
                    return Err(ParseApiKeyError::EmptyKeyId);
                }

                if secret.is_empty() {
                    return Err(ParseApiKeyError::EmptyKeySecret);
                }

                Ok(ApiKey::new(key_id, secret))
            }
            None => Err(ParseApiKeyError::MissingColon),
        }
    }
}

impl<'de> serde::Deserialize<'de> for ApiKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum ApiKeyDeserializeHelper {
            String(String),
            Struct { id: ApiKeyId, secret: ApiKeySecret },
        }

        match ApiKeyDeserializeHelper::deserialize(deserializer).map_err(|_| {
            serde::de::Error::custom(
                "\
                Failed to deserialize API key\n\
                Expected structure with fields 'id' and 'secret' or string with format '<id>:<secret>'\
                ",
            )
        })? {
            ApiKeyDeserializeHelper::String(s) => ApiKey::from_str(&s)
                .map_err(|e| serde::de::Error::custom(e.to_string())),
            ApiKeyDeserializeHelper::Struct { id, secret } => {
                if id.0.is_empty() {
                    return Err(<D::Error as serde::de::Error>::custom(ParseApiKeyError::EmptyKeyId.to_string()));
                }

                if secret.0.is_empty() {
                    return Err(<D::Error as serde::de::Error>::custom(ParseApiKeyError::EmptyKeySecret.to_string()));
                }

                Ok(ApiKey { id, secret })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::api_key::ApiKey;

    #[derive(Deserialize)]
    struct Test {
        api_key: ApiKey,
    }

    #[test]
    fn deserialize_str() {
        let toml = r#"        
        api_key = "roomserver:secret"
        "#;

        let Test { api_key } = match toml::from_str(toml) {
            Ok(c) => c,
            Err(e) => panic!("{e}"),
        };

        assert_eq!(api_key, ApiKey::new("roomserver", "secret"))
    }

    #[test]
    fn deserialize_missing_key_str() {
        let toml = r#"        
        api_key = ":secret"
        "#;

        if toml::from_str::<Test>(toml).is_ok() {
            panic!("Expected error")
        };
    }

    #[test]
    fn deserialize_missing_secret_str() {
        let toml = r#"        
        api_key = "key_id:"
        "#;

        if toml::from_str::<Test>(toml).is_ok() {
            panic!("Expected error")
        };
    }

    #[test]
    fn deserialize_missing_colon_str() {
        let toml = r#"        
        api_key = "key_id:"
        "#;

        if toml::from_str::<Test>(toml).is_ok() {
            panic!("Expected error")
        };
    }

    #[test]
    fn deserialize_struct() {
        let toml = r#"        
        [api_key]
            id = "roomserver"
            secret = "secret"
        "#;

        let Test { api_key } = match toml::from_str(toml) {
            Ok(c) => c,
            Err(e) => panic!("{e}"),
        };

        assert_eq!(api_key, ApiKey::new("roomserver", "secret"))
    }
}

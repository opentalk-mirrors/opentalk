// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::HashMap;

use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::Deserialize;

use crate::ApiKeyId;

#[derive(Debug, thiserror::Error)]
pub(crate) enum DecodingError {
    #[error("Missing key id in JWT")]
    MissingKeyId,
    #[error("Unknown key id in JWT")]
    UnknownKeyId,
    #[error("Failed to parse JWT")]
    MalformedJwt,
    #[error(transparent)]
    JwtError(#[from] jsonwebtoken::errors::Error),
}

/// Placeholder struct for decoding JWT
#[derive(Debug, Clone, Deserialize)]
struct Empty {}

#[derive(Debug, Clone)]
pub(crate) struct DecodingKeys {
    keys: HashMap<ApiKeyId, DecodingKey>,
}

impl DecodingKeys {
    pub(crate) fn new(keys: HashMap<ApiKeyId, DecodingKey>) -> Self {
        Self { keys }
    }

    /// Validate the signature and expiry of the provided JWT
    pub(crate) fn validate_jwt(&self, jwt: &[u8]) -> Result<(), DecodingError> {
        let header = jsonwebtoken::decode_header(jwt)?;

        let key_id = header.kid.ok_or(DecodingError::MissingKeyId)?;

        let decoding_key = self
            .keys
            .get(&ApiKeyId(key_id))
            .ok_or(DecodingError::UnknownKeyId)?;

        let _ = decode::<Empty>(jwt, decoding_key, &Validation::default())?;

        Ok(())
    }
}

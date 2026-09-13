// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This library provides middlewares to authorize internal services

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

#[cfg(any(feature = "actix", feature = "axum"))]
pub mod service;

mod api_key;

pub use api_key::{ApiKey, EncodingError, JWT_EXPIRY};

/// The identifier of an API key
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ApiKeyId(pub String);

impl<T: Into<String>> From<T> for ApiKeyId {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for ApiKeyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

/// The API key secret
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ApiKeySecret(pub String);

impl<T: Into<String>> From<T> for ApiKeySecret {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl Debug for ApiKeySecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Key").field(&"[redacted]").finish()
    }
}

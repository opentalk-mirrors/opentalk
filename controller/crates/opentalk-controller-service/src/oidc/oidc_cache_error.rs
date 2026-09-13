// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::TimeDelta;
use opentalk_cache::CacheError;
use opentalk_controller_utils::CaptureApiError;
use opentalk_types_api_v1::error::{ApiError, AuthenticationError};
use snafu::Snafu;

use super::cacheable::DecodeFromCacheError;

/// OIDC cache errors
#[derive(Debug, Snafu)]
pub enum OidcCacheError {
    /// Underlying cache storage errors
    #[snafu(display("cache error: {source}"))]
    Cache {
        /// The source error from the cache storage
        source: CacheError,
    },

    /// Errors when decoding cached values
    #[snafu(display("cache error while decoding: {source}"))]
    DecodeFromCacheError {
        /// The source error of the decoding operation
        source: DecodeFromCacheError,
    },

    /// Error on attempt to cache a token, which expires too soon
    #[snafu(display("token expires soon and will not be cached (ttl={ttl:?})"))]
    TokenTtlTooShort {
        /// Provided token TTL, which was considered too short
        ttl: TimeDelta,
    },

    /// Error on attempt to cache a token, which has no expiry date
    #[snafu(display("token has no expiry and will not be cached"))]
    NoExpiryForToken,

    /// Error on attempt to update a token, which do not exist in the cache yet
    #[snafu(display("token doen't exist in cache and cannot be updated"))]
    CannotUpdateNonExistingToken,

    /// Error indicating that a formerly valid access token has been revoked by sub logout
    #[snafu(display("token has been revoked by sub logout"))]
    RevokedByLogout,

    /// Error on attmept to cache an internal error for an access token
    /// This is not allowed, because such errors are transient and do not relate to the token validity
    #[snafu(display("internal errors will not be cached for acess tokens"))]
    NoCachingOfInternalErrors,
}

impl From<CacheError> for OidcCacheError {
    fn from(source: CacheError) -> Self {
        Self::Cache { source }
    }
}

impl From<DecodeFromCacheError> for OidcCacheError {
    fn from(source: DecodeFromCacheError) -> Self {
        Self::DecodeFromCacheError { source }
    }
}

impl From<OidcCacheError> for CaptureApiError {
    fn from(source: OidcCacheError) -> CaptureApiError {
        match source {
            OidcCacheError::Cache { .. }
            | OidcCacheError::DecodeFromCacheError { .. }
            | OidcCacheError::TokenTtlTooShort { .. }
            | OidcCacheError::NoCachingOfInternalErrors
            | OidcCacheError::CannotUpdateNonExistingToken => {
                CaptureApiError::from(ApiError::internal().with_message(source.to_string()))
            }
            OidcCacheError::RevokedByLogout => CaptureApiError::from(
                ApiError::unauthorized()
                    .with_www_authenticate(AuthenticationError::AccessTokenInactive)
                    .with_code("revoked_token")
                    .with_message(source.to_string()),
            ),
            OidcCacheError::NoExpiryForToken => CaptureApiError::from(
                ApiError::unauthorized()
                    .with_www_authenticate(AuthenticationError::InvalidAccessToken)
                    .with_code("invalid_token")
                    .with_message("access token has no expiry and considered invalid"),
            ),
        }
    }
}

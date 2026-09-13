// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use core::time::Duration;
use std::{fmt::Display, hash::Hash};

use chrono::{DateTime, Utc};
use openidconnect::{AccessToken, SubjectIdentifier};
use opentalk_cache::{
    CacheStorage, CacheUpdateMode, hashing::WithHashing, local, overlay::WithOverlay, redis,
};
use opentalk_controller_utils::CaptureApiError;
use opentalk_inventory::{Tenant, User};
use snafu::Report;

use super::{
    LogoutMarker, OidcCacheError,
    cacheable::{
        AccessTokenResult, ApiError as CacheableApiError, Tenant as CacheableTenant,
        User as CacheableUser,
    },
};
use crate::RedisConnection;

pub type Result<T, E = CaptureApiError> = std::result::Result<T, E>;

use opentalk_types_api_v1::error::ApiError;

const ACCESS_TOKEN_DEFAULT_TTL_SECS: u64 = 60 * 5;
/// Must be much longer, than for access tokens
const SUB_LOGOUT_MARKERS_DEFAULT_TTL_SECS: u64 = 60 * 60 * 2;
const MIN_TOKEN_TTL_SECS: i64 = 10;

/// Cache for OpenID Connect related data
pub struct Cache {
    /// Cache storage for access tokens
    pub access_tokens: Box<dyn CacheStorage<String, AccessTokenResult> + Send + Sync>,
    /// Cache storage for logout markers of the OIDC subjects
    pub sub_logout_markers: Box<dyn CacheStorage<String, LogoutMarker> + Send + Sync>,
}

impl Cache {
    /// Create a new [`Cache`] instance with an optional [`RedisConnection`].
    pub fn create(redis: Option<RedisConnection>) -> Self {
        Self {
            access_tokens: Self::build_cache_with_hashing(
                redis.clone(),
                "user-access-tokens".to_string(),
                Duration::from_secs(ACCESS_TOKEN_DEFAULT_TTL_SECS),
                CacheUpdateMode::KeepTtl,
            ),
            sub_logout_markers: Self::build_cache(
                redis,
                "sub-logout-markers".to_string(),
                Duration::from_secs(SUB_LOGOUT_MARKERS_DEFAULT_TTL_SECS),
                CacheUpdateMode::ResetTtl,
            ),
        }
    }

    /// Build a cache storage
    /// If Redis connection is available, combine a Redis cache with a local in-memory cache as an overlay
    /// Otherise only local cache is used
    fn build_cache<K, V>(
        redis: Option<RedisConnection>,
        prefix: String,
        ttl: Duration,
        mode: CacheUpdateMode,
    ) -> Box<dyn CacheStorage<K, V> + Send + Sync>
    where
        K: redis::Key + local::Key + Clone + Display + Hash + 'static,
        V: redis::Value + local::Value + 'static,
    {
        let local_cache = local::Cache::new(ttl, mode);

        let Some(redis) = redis else {
            return Box::new(local_cache);
        };
        let redis = redis.into_manager();

        let redis_cache = redis::Cache::new(redis, prefix, ttl, mode);

        Box::new(redis_cache.with_overlay(local_cache))
    }

    /// Build a cache storage with key hashing to reduce memory usage for large keys
    /// If Redis connection is available, compose a Redis cache with a local in-memory cache as an overly
    /// Otherise only local cache is used
    fn build_cache_with_hashing<K, V>(
        redis: Option<RedisConnection>,
        prefix: String,
        ttl: Duration,
        mode: CacheUpdateMode,
    ) -> Box<dyn CacheStorage<K, V> + Send + Sync>
    where
        K: redis::Key + local::Key + Clone + Display + Hash + 'static,
        V: redis::Value + local::Value + 'static,
    {
        let local_cache = local::Cache::new(ttl, mode);

        let Some(redis) = redis else {
            return Box::new(local_cache.with_hashing());
        };
        let redis = redis.into_manager();

        let redis_cache = redis::Cache::new(redis, prefix, ttl, mode);

        Box::new(redis_cache.with_overlay(local_cache).with_hashing())
    }

    /// Insert an access token into the cache with a specific expiry date
    /// Cache will reject a token, which has no expiry date or its ttl is too short
    /// Value is either a valid token metadata or an error
    pub async fn insert_access_token(
        &self,
        access_token: &AccessToken,
        value: Result<(Tenant, User), CaptureApiError>,
        maybe_expires_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        // If an error passed, we just cache without additional checks
        let Ok(value) = value else {
            return self
                .cache_access_token_error(access_token, value.unwrap_err())
                .await;
        };

        // Don't cache tokens without expiration
        let Some(expires_at) = maybe_expires_at else {
            let error = CaptureApiError::from(OidcCacheError::NoExpiryForToken);
            // Even if caching will return error, we still want to return the original error to the caller, as it is more relevant
            let _ = self
                .cache_access_token_error(access_token, error.clone())
                .await;
            return Err(error);
        };

        // Don't cache tokens that expire too soon
        let token_ttl = expires_at - Utc::now();
        if token_ttl <= chrono::Duration::seconds(MIN_TOKEN_TTL_SECS) {
            return Err(CaptureApiError::from(OidcCacheError::TokenTtlTooShort {
                ttl: token_ttl,
            }));
        }

        // Add logout marker to enable lazy invalidation on backchannel logout
        let (tenant, user) = value;
        let logout_marker = self.calculate_logout_marker(&user.oidc_sub).await?;
        let value = Ok((
            CacheableTenant::from(tenant),
            CacheableUser::from(user),
            logout_marker,
        ));

        self.access_tokens
            .insert_with_ttl(
                access_token.secret().clone(),
                value,
                token_ttl.to_std().expect("duration was previously checked"),
            )
            .await
            .map_err(CaptureApiError::from)
    }

    /// Updates value for a valid cached access token
    /// Cache will reject a token, which do not exist in the cache yet
    /// On update the original TTL of the cache entry will be kept
    /// Value is either a valid token metadata or an error
    pub async fn update_access_token(
        &self,
        access_token: &AccessToken,
        value: Result<(Tenant, User), CaptureApiError>,
    ) -> Result<()> {
        let value = match value {
            Ok((tenant, user)) => Ok((CacheableTenant::from(tenant), CacheableUser::from(user))),
            Err(e) => Err(CacheableApiError::from(e)),
        };

        let cached = match self.access_tokens.get(access_token.secret()).await {
            Ok(Some(result)) => result,
            Ok(None) => {
                return Err(CaptureApiError::from(
                    OidcCacheError::CannotUpdateNonExistingToken,
                ));
            }
            Err(e) => return Err(CaptureApiError::from(e)),
        };

        // For valid token we need to preserve the logout marker
        let value_with_logout = match cached {
            Ok((_, _, logout_marker)) => value.map(|(tenant, user)| (tenant, user, logout_marker)),
            Err(e) => Err(e),
        };

        // We know the token exists in the cache, so we can safely update it with default TTL
        // The cache is configured to keep original TTL on update
        self.access_tokens
            .insert(access_token.secret().clone(), value_with_logout)
            .await
            .map_err(CaptureApiError::from)
    }

    /// Lazy access token invalidation based on sub logout
    /// If an access token has been cached before it's assoiciated sub has been logged out
    /// it is considered as revoked
    async fn is_access_token_revoked_by_sub_logout(
        &self,
        token_logout_marker: LogoutMarker,
        sub: &String,
    ) -> Result<bool> {
        match self.sub_logout_markers.get(sub).await {
            Ok(Some(sub_logout_marker)) => {
                Ok(token_logout_marker.value() <= sub_logout_marker.value())
            }
            Ok(None) => Ok(false),
            Err(e) => {
                log::warn!(
                    "Failed to retreive logout marker from the cache, error: {}",
                    Report::from_error(&e)
                );
                Err(CaptureApiError::from(e))
            }
        }
    }

    /// Get cached result for an access token
    /// Function also performs lazy invalidation of the access token
    /// If the cached token has been revoked by backchannel logout, it will update the entry with the error
    pub async fn get_access_token(
        &self,
        access_token: &AccessToken,
    ) -> Option<Result<(Tenant, User), CaptureApiError>> {
        match self.access_tokens.get(access_token.secret()).await {
            Ok(Some(Ok((tenant, user, token_logout_marker)))) => {
                let is_revoked = self
                    .is_access_token_revoked_by_sub_logout(token_logout_marker, user.oidc_sub())
                    .await;
                if let Ok(true) = is_revoked {
                    let error = CaptureApiError::from(OidcCacheError::RevokedByLogout);
                    // Even if caching will return error, we still want to return the original error to the caller, as it is more relevant
                    let _ = self
                        .cache_access_token_error(access_token, error.clone())
                        .await;
                    return Some(Err(error));
                }

                let tenant = match Tenant::try_from(tenant) {
                    Ok(tenant) => tenant,
                    Err(e) => {
                        log::warn!(
                            "Error when attempting to read tenant information loaded from token cache: {}",
                            Report::from_error(&e)
                        );
                        return Some(Err(OidcCacheError::from(e).into()));
                    }
                };

                let user = match User::try_from(user) {
                    Ok(tenant) => tenant,
                    Err(e) => {
                        log::warn!(
                            "Error when attempting to read user information loaded from token cache: {}",
                            Report::from_error(&e)
                        );
                        return Some(Err(OidcCacheError::from(e).into()));
                    }
                };

                Some(Ok((tenant, user)))
            }
            Ok(Some(Err(cached_error))) => {
                let cached_error = match CaptureApiError::try_from(cached_error) {
                    Ok(cached_error) => cached_error,
                    Err(e) => {
                        log::warn!(
                            "Error when attempting to read erroneous verification result loaded from token cache: {}",
                            Report::from_error(&e)
                        );
                        return Some(Err(OidcCacheError::from(e).into()));
                    }
                };

                Some(Err(cached_error))
            }
            Ok(None) => None,
            Err(e) => Some(Err(e.into())),
        }
    }

    /// Insert logout marker for a specific OIDC subject
    async fn insert_sub_logout_marker(&self, sub: String, marker: LogoutMarker) -> Result<()> {
        self.sub_logout_markers
            .insert(sub, marker)
            .await
            .map_err(|e| {
                log::warn!(
                    "Failed to cache logout marker, error: {}",
                    Report::from_error(&e)
                );
                CaptureApiError::from(e)
            })
    }

    /// Insert or update logout marker for a specific OIDC subject in the cache
    /// to invalidate all existing access tokens for the subject
    pub async fn upsert_sub_logout_marker(&self, sub: SubjectIdentifier) -> Result<()> {
        let sub = String::from(sub);
        let logout_marker = self.calculate_logout_marker(&sub).await?;
        self.insert_sub_logout_marker(sub, logout_marker).await?;
        Ok(())
    }

    /// Calculate logout marker for the associated OIDC subject
    /// If marker for the subject does not exist: insert a new one with value 0
    /// If marker for the subject exists: increment the marker
    async fn calculate_logout_marker(&self, sub: &String) -> Result<LogoutMarker> {
        match self.sub_logout_markers.get(sub).await {
            Ok(Some(mut marker)) => {
                marker.increment();
                Ok(marker)
            }
            Ok(None) => Ok(LogoutMarker::from(0)),
            Err(e) => {
                log::warn!(
                    "Failed to retreive logout marker from the cache, error: {}",
                    Report::from_error(&e)
                );
                Err(CaptureApiError::from(e))
            }
        }
    }

    /// Caches access token errors such as verification errors or inactive tokens
    /// Internal errors will not be cached, as they can be relevant and not related to the token validity
    async fn cache_access_token_error(
        &self,
        access_token: &AccessToken,
        error: CaptureApiError,
    ) -> Result<()> {
        if ApiError::from(error.clone()).status != 500 {
            self.access_tokens
                .insert(
                    access_token.secret().clone(),
                    Err(CacheableApiError::from(error)),
                )
                .await
                .map_err(|e| {
                    log::warn!("Failed to cache access token error: {e}");
                    CaptureApiError::from(e)
                })
        } else {
            log::warn!("Internal errors will not be cached for acess tokens");
            Err(CaptureApiError::from(
                OidcCacheError::NoCachingOfInternalErrors,
            ))
        }
    }
}

impl std::fmt::Debug for Cache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Caches")
    }
}

/// We are testing only local cache without Redis connection
#[cfg(test)]
mod tests {
    use opentalk_types_common::utils::ExampleData;
    use tokio;

    use super::*;

    fn create_cache() -> Cache {
        Cache::create(None)
    }

    fn create_access_token() -> AccessToken {
        AccessToken::new(String::from("access-token-1"))
    }

    fn create_expiration() -> Option<DateTime<Utc>> {
        Some(Utc::now() + chrono::Duration::seconds(MIN_TOKEN_TTL_SECS + 1))
    }

    fn create_user() -> User {
        User::example_data()
    }

    fn create_tenant() -> Tenant {
        Tenant::example_data()
    }

    fn create_valid_value() -> Result<(Tenant, User), CaptureApiError> {
        Ok((create_tenant(), create_user()))
    }

    fn create_sub(sub: &str) -> SubjectIdentifier {
        SubjectIdentifier::new(sub.to_owned())
    }

    #[tokio::test]
    async fn insert_access_token_successfully() {
        let cache = create_cache();
        let access_token = create_access_token();
        let expires_at = create_expiration();
        let value = create_valid_value();

        let result = cache
            .insert_access_token(&access_token, value, expires_at)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn insert_access_token_value_with_no_expiration_failed() {
        let cache = create_cache();
        let access_token = create_access_token();
        let expires_at = None;
        let value = create_valid_value();

        let result = cache
            .insert_access_token(&access_token, value, expires_at)
            .await;
        assert!(
            result
                .err()
                .map(|e| e.to_string().contains("token has no expiry"))
                .unwrap_or(false)
        );
    }

    #[tokio::test]
    async fn insert_access_token_error_with_no_expiration_successfully() {
        let cache = create_cache();
        let access_token = create_access_token();
        let expires_at = None;
        let error = Err(CaptureApiError::from(OidcCacheError::NoExpiryForToken));

        let result = cache
            .insert_access_token(&access_token, error, expires_at)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn insert_access_token_internal_error_failed() {
        let cache = create_cache();
        let access_token = create_access_token();
        let expires_at = None;
        let error = Err(CaptureApiError::from(ApiError::internal()));

        let result = cache
            .insert_access_token(&access_token, error, expires_at)
            .await;
        assert!(
            result
                .err()
                .map(|e| e
                    .to_string()
                    .contains("internal errors will not be cached for acess tokens"))
                .unwrap_or(false)
        );
    }

    #[tokio::test]
    async fn insert_access_token_with_short_expiration_failed() {
        let cache = create_cache();
        let access_token = create_access_token();
        let expires_at = Some(Utc::now() + chrono::Duration::seconds(MIN_TOKEN_TTL_SECS - 1));
        let value = create_valid_value();

        let result = cache
            .insert_access_token(&access_token, value, expires_at)
            .await;
        assert!(
            result
                .err()
                .map(|e| e
                    .to_string()
                    .contains("token expires soon and will not be cached"))
                .unwrap_or(false)
        );
    }

    #[tokio::test]
    async fn update_access_token_for_existing_token_successfully() {
        let cache = create_cache();
        let access_token = create_access_token();
        let expires_at = create_expiration();
        let mut value = create_valid_value();

        cache
            .insert_access_token(&access_token, value.clone(), expires_at)
            .await
            .expect("Failed to insert access token");

        // Update value for the existing token
        let update_name = "UpdateName".to_string();
        value.as_mut().unwrap().1.firstname = update_name.clone();
        let result = cache.update_access_token(&access_token, value).await;
        assert!(result.is_ok());

        // Check updated value
        let updated_value = cache
            .get_access_token(&access_token)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_value.1.firstname, update_name)
    }

    #[tokio::test]
    async fn update_access_token_for_nonexisting_token_failed() {
        let cache = create_cache();
        let access_token = create_access_token();
        let value = create_valid_value();

        let result = cache.update_access_token(&access_token, value).await;
        assert!(
            result
                .err()
                .map(|e| e
                    .to_string()
                    .contains("token doen't exist in cache and cannot be updated"))
                .unwrap_or(false)
        );
    }

    #[tokio::test]
    async fn get_access_token_for_existing_token_successfully() {
        let cache = create_cache();
        let access_token = create_access_token();
        let expires_at = create_expiration();
        let value = create_valid_value();

        cache
            .insert_access_token(&access_token, value.clone(), expires_at)
            .await
            .expect("Failed to insert access token");

        let result = cache.get_access_token(&access_token).await;
        assert_ne!(None, result);
    }

    #[tokio::test]
    async fn get_access_token_for_nonexisting_token_successfully() {
        let cache = create_cache();
        let access_token = create_access_token();
        let result = cache.get_access_token(&access_token).await;
        assert_eq!(None, result);
    }

    #[tokio::test]
    // If an access token has been cached before it's associated sub has been logged out
    // it is considered as revoked and cache should return an error when attempting to get it
    async fn get_access_token_cached_before_sub_logout_failed() {
        let cache = create_cache();
        let access_token = create_access_token();
        let expires_at = create_expiration();
        let value = create_valid_value();

        let _ = cache
            .insert_access_token(&access_token, value.clone(), expires_at)
            .await;

        let sub = create_sub(&value.as_ref().unwrap().1.oidc_sub);
        let _ = cache.upsert_sub_logout_marker(sub).await;

        let result = cache.get_access_token(&access_token).await.unwrap();
        assert!(
            result
                .err()
                .map(|e| e
                    .to_string()
                    .contains("token has been revoked by sub logout"))
                .unwrap_or(false)
        );
    }

    #[tokio::test]
    // If an access token has been cached after it's associated sub has been logged out
    // it should be considered as valid and cache should return it successfully
    async fn get_access_token_cached_after_sub_logout_successfully() {
        let cache = create_cache();
        let access_token = create_access_token();
        let expires_at = create_expiration();
        let value = create_valid_value();

        let sub = create_sub(&value.as_ref().unwrap().1.oidc_sub);
        let _ = cache.upsert_sub_logout_marker(sub).await;

        let _ = cache
            .insert_access_token(&access_token, value.clone(), expires_at)
            .await;

        let result = cache.get_access_token(&access_token).await.unwrap();
        assert!(result.is_ok());
    }
}

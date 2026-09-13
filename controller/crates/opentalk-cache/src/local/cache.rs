// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use super::{Entry, EntryExpiry, Key, Value};
use crate::{CacheStorage, CacheUpdateMode, Result};

pub struct Cache<K, V> {
    ttl: Duration,
    inner: moka::future::Cache<K, Entry<V>>,
    mode: CacheUpdateMode,
}

impl<K, V> Cache<K, V>
where
    K: Key + 'static,
    V: Value + 'static,
{
    pub fn new(ttl: Duration, mode: CacheUpdateMode) -> Self {
        Self {
            ttl,
            inner: moka::future::Cache::builder()
                .time_to_live(ttl)
                .expire_after(EntryExpiry)
                .build(),
            mode,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<K, V> CacheStorage<K, V> for Cache<K, V>
where
    K: Key + 'static,
    V: Value + 'static,
{
    fn ttl(&self) -> Duration {
        self.ttl
    }

    async fn get(&self, key: &K) -> Result<Option<V>> {
        Ok(self.inner.get(key).await.map(|entry| entry.into_inner()))
    }

    async fn insert(&self, key: K, value: V) -> Result<()> {
        self.inner
            .insert(key, Entry::new(value, self.ttl, self.mode))
            .await;
        Ok(())
    }

    async fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) -> Result<()> {
        self.inner
            .insert(key, Entry::new(value, ttl, self.mode))
            .await;
        Ok(())
    }

    async fn invalidate(&self, key: &K) -> Result<()> {
        self.inner.invalidate(key).await;
        Ok(())
    }
}

/// TODO: We could make these tests more generic and move them up to the lib level
///       to test all CacheStorage implementations (redis, local)
#[cfg(test)]
mod tests {
    use tokio;

    use super::*;

    const DEFAULT_TTL: Duration = Duration::from_millis(300);
    const CUSTOM_SHORTER_TTL: Duration = Duration::from_millis(100);
    const CUSTOM_LONGER_TTL: Duration = Duration::from_millis(500);
    const DEFAULT_CACHE_UPDATE_MODE: CacheUpdateMode = CacheUpdateMode::ResetTtl;

    async fn setup() -> (Cache<String, String>, String, String) {
        let cache = Cache::new(DEFAULT_TTL, DEFAULT_CACHE_UPDATE_MODE);
        let key = String::from("key");
        let value = String::from("value");

        cache.insert(key.clone(), value.clone()).await.unwrap();
        (cache, key, value)
    }

    async fn setup_with_entry_ttl(ttl: Duration) -> (Cache<String, String>, String, String) {
        let cache = Cache::new(DEFAULT_TTL, DEFAULT_CACHE_UPDATE_MODE);
        let key = String::from("key");
        let value = String::from("value");

        cache
            .insert_with_ttl(key.clone(), value.clone(), ttl)
            .await
            .unwrap();
        (cache, key, value)
    }

    #[tokio::test]
    async fn insertion() {
        let (cache, original_key, original_value) = setup().await;

        let cached_value = cache.get(&original_key).await.unwrap();
        assert_eq!(Some(original_value), cached_value);
    }

    #[tokio::test]
    async fn invalidation() {
        let (cache, original_key, _original_value) = setup().await;

        cache.invalidate(&original_key).await.unwrap();
        let cached_value = cache.get(&original_key).await.unwrap();
        assert_eq!(cached_value, None);
    }

    #[tokio::test]
    async fn expiration_with_default_ttl() {
        let (cache, original_key, _original_value) = setup().await;

        tokio::time::sleep(DEFAULT_TTL).await;

        let cached_value = cache.get(&original_key).await.unwrap();
        assert_eq!(cached_value, None);
    }

    #[tokio::test]
    async fn expiration_with_shorter_ttl() {
        let (cache, original_key, _original_value) = setup_with_entry_ttl(CUSTOM_SHORTER_TTL).await;

        tokio::time::sleep(CUSTOM_SHORTER_TTL).await;

        let cached_value = cache.get(&original_key).await.unwrap();
        assert_eq!(cached_value, None);
    }

    #[tokio::test]
    async fn expiration_with_longer_ttl() {
        let (cache, original_key, _original_value) = setup_with_entry_ttl(CUSTOM_LONGER_TTL).await;

        tokio::time::sleep(DEFAULT_TTL).await;

        let cached_value = cache.get(&original_key).await.unwrap();
        assert_eq!(cached_value, None);
    }

    #[tokio::test]
    async fn reset_ttl_on_entry_update() {
        let (cache, original_key, _original_value) = setup().await;

        let delta = Duration::from_millis(50);
        tokio::time::sleep(DEFAULT_TTL - delta).await;

        // Update the cache entry before expiration -> this should reset the TTL
        let new_value = String::from("new value");
        cache
            .insert(original_key.clone(), new_value.clone())
            .await
            .unwrap();
        tokio::time::sleep(delta * 2).await;

        // Time a bit longer than default TTL has passed since first insertion, but the entry should still be available
        let cached_value = cache.get(&original_key).await.unwrap();
        assert_eq!(Some(new_value), cached_value);

        // Time a bit longer than default TTL has passed since the update, now the entry should be expired
        tokio::time::sleep(DEFAULT_TTL).await;
        let cached_value = cache.get(&original_key).await.unwrap();
        assert_eq!(cached_value, None);
    }

    #[tokio::test]
    async fn keep_ttl_on_entry_update() {
        let cache = Cache::new(DEFAULT_TTL, CacheUpdateMode::KeepTtl);
        let key = String::from("key");
        let value = String::from("value");

        cache.insert(key.clone(), value.clone()).await.unwrap();

        let delta = Duration::from_millis(50);
        tokio::time::sleep(DEFAULT_TTL - delta).await;

        // Update the cache entry before expiration -> this should keep the original TTL
        let new_value = String::from("new value");
        cache.insert(key.clone(), new_value).await.unwrap();
        tokio::time::sleep(delta * 2).await;

        // Wait a bit longer than default TTL has passed since first insertion, the value should be expired
        let cached_value = cache.get(&key).await.unwrap();
        assert_eq!(cached_value, None);
    }
}

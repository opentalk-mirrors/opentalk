// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2
use std::{hash::Hash, marker::PhantomData, time::Duration};

use siphasher::sip128::{Hasher128, SipHasher24};

use crate::{CacheStorage, Result};

pub struct HashedCache<K, V, C>
where
    K: Hash,
    C: CacheStorage<u128, V>,
{
    inner: C,
    _phantom: PhantomData<(K, V)>,
}

impl<K, V, C> HashedCache<K, V, C>
where
    K: Hash,
    C: CacheStorage<u128, V>,
{
    pub fn new(inner: C) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    pub fn hash_key(&self, raw: &K) -> u128 {
        let mut h = SipHasher24::new_with_keys(!0x113, 0x311);
        raw.hash(&mut h);
        h.finish128().as_u128()
    }
}

pub trait WithHashing<K, V>: CacheStorage<u128, V> + Sized {
    fn with_hashing(self) -> HashedCache<K, V, Self>
    where
        K: Hash;
}

impl<K, V, CACHE: CacheStorage<u128, V>> WithHashing<K, V> for CACHE
where
    K: Hash,
{
    fn with_hashing(self) -> HashedCache<K, V, Self> {
        HashedCache::new(self)
    }
}

#[async_trait::async_trait(?Send)]
impl<K, V, C> CacheStorage<K, V> for HashedCache<K, V, C>
where
    K: Hash + 'static,
    V: Clone + 'static,
    C: CacheStorage<u128, V>,
{
    fn ttl(&self) -> Duration {
        self.inner.ttl()
    }

    async fn get(&self, key: &K) -> Result<Option<V>> {
        let hashed_key = self.hash_key(key);
        self.inner.get(&hashed_key).await
    }

    async fn insert(&self, key: K, value: V) -> Result<()> {
        let hashed_key = self.hash_key(&key);
        self.inner.insert(hashed_key, value).await
    }

    async fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) -> Result<()> {
        let hashed_key = self.hash_key(&key);
        self.inner.insert_with_ttl(hashed_key, value, ttl).await
    }

    async fn invalidate(&self, key: &K) -> Result<()> {
        let hashed_key = self.hash_key(key);
        self.inner.invalidate(&hashed_key).await
    }
}

#[cfg(test)]
mod tests {
    use tokio;

    use super::*;
    use crate::{CacheUpdateMode, local::Cache, overlay::WithOverlay};

    const DEFAULT_TTL: Duration = Duration::from_millis(300);
    const DEFAULT_CACHE_UPDATE_MODE: CacheUpdateMode = CacheUpdateMode::ResetTtl;
    const JWT_KEY: &str = "eyJ3bGciOiJSUzI1NiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICJNaFJKdWt5LU5BQjFiM21kNWZrUDI4NU9CZ08wVTBoMmRlYTdvdHZicDhjIn0.eyJleHAiOjE3NjkzNjQ4NzUsImlhdCI6MTc2OTM2NDU3NSwiYXV0aF90aW1lIjoxNzY5MjQxNTY1LCJqdGkiOiJkMGEwZjA1NC1mNzA5LTRlYTQtODRhOS00YTZjYmUyNzA0MTEiLCJpc3MiOiJodHRwczovL2FjY291bnRzLm9wZW50YWxrLmRldi9hdXRoL3JlYWxtcy9oZWlubGVpbi12aWRlbyIsImF1ZCI6ImFjY291bnQiLCJzdWIiOiI4OWM1NTVmZC1lNGZjLTRlODktYWNjYi1jM2IwMmY4MzE0MzUiLCJ0eXAiOiJCZWFyZXIiLCJhenAiOiJvcGVuaWRfb3BlbnRhbGtfc3RhZ2luZ19wdWJsaWMiLCJzZXNzaW9uX3N0YXRlIjoiMjdkODUwOWYtMGQ3MC00MjRiLTlkOTEtZGMxMjNhMjU0OGIwIiwiYWxsb3dlZC1vcmlnaW5zIjpbImh0dHBzOi8vc3RhZ2luZy5oZWlubGVpbi12aWRlby5kZSIsImh0dHBzOi8vc3RhZ2luZy5vcGVudGFsay5ydW4iLCJodHRwOi8vbG9jYWxob3N0OjMwMDAiXSwicmVhbG1fYWNjZXNzIjp7InJvbGVzIjpbImRlZmF1bHQtcm9sZXMtaGVpbmxlaW4tdmlkZW8iLCJvZmZsaW5lX2FjY2VzcyIsIlN0YWZmIiwidW1hX2F1dGhvcml6YXRpb24iXX0sInJlc291cmNlX2FjY2VzcyI6eyJhY2NvdW50Ijp7InJvbGVzIjpbIm1hbmFnZS1hY2NvdW50IiwibWFuYWdlLWFjY291bnQtbGlua3MiLCJ2aWV3LXByb2ZpbGUiXX19LCJzY29wZSI6Im9wZW5pZCB0ZW5hbmN5IHByb2ZpbGUgdGFyaWZmIGVtYWlsIiwic2lkIjoiMjdkODUwOWYtMGQ3MC00MjRiLTlkOTEtZGMxMjNhMjU0OGIwIiwidGVuYW50X2lkIjoiT3BlblRhbGtEZWZhdWx0VGVuYW50IiwidGFyaWZmX2lkIjoiT3BlblRhbGtEZWZhdWx0VGFyaWZmIiwiZW1haWxfdmVyaWZpZWQiOnRydWUsIm5hbWUiOiJvbGVrc2lpIHN1a2hvZG9sc2t5aSIsInhfZ3JwIjpbIi9IZWlubGVpbi1WaWRlbyBTdGFmZiJdLCJwcmVmZXJyZWRfdXNlcm5hbWUiOiJvLnN1a2hvZG9sc2t5aSIsImdpdmVuX25hbWUiOiJvbGVrc2lpIiwiZmFtaWx5X25hbWUiOiJzdWtob2RvbHNreWkiLCJlbWFpbCI6Im8uc3VraG9kb2xza3lpQG9wZW50YWxrLmV1In0.cubK08iRQOz8YH0QXoSqF_Fc7kHOclhZ_hrcoG8VO7OAd3yEbRGoXOc5IfmwhYXLMFvBkBufbhFbv55_pS94dfk5yAN2VadnWRlxJ4lmboa5KWoXIkvTuhzjlyvVSXUVaLxmtSspQ1aQKkwDuJPeVXFanbYiT12oJlW1TQ8eFhZYFZO8VyyS93532CK-OS3a4n9QAAiqL2pV4bpoNCQEA67CpY5zyucxAP34-5uMm6qLNS9XQys1fDZlDFXmSkGcW6LeZwQ24edt1m8mdalLeolFKrbYwe1m78nPBVZopAA2ie5LWZAWJYdPyLMj9IOzUKyHe41a6U91efcRPYaMbA";
    const VALUE: &str = "value";

    #[tokio::test]
    async fn insertion() {
        let cache = Cache::new(DEFAULT_TTL, DEFAULT_CACHE_UPDATE_MODE).with_hashing();

        let raw_key = String::from(JWT_KEY);
        let value = String::from(VALUE);
        cache.insert(raw_key, value).await.unwrap();

        // Inner cache should contain the hashed key
        let raw_key = String::from(JWT_KEY);
        let hashed_key = cache.hash_key(&raw_key);
        let result = cache.inner.get(&hashed_key).await.unwrap();
        assert_eq!(result, Some(String::from(VALUE)));

        // Getting via the usual API should also work
        let result = cache.get(&raw_key).await.unwrap();
        assert_eq!(result, Some(String::from(VALUE)));
    }

    #[tokio::test]
    async fn insertion_overlay() {
        let base: Cache<u128, String> = Cache::new(DEFAULT_TTL, DEFAULT_CACHE_UPDATE_MODE);
        let overlay: Cache<u128, String> = Cache::new(DEFAULT_TTL, DEFAULT_CACHE_UPDATE_MODE);
        let cache = base.with_overlay(overlay).with_hashing();

        let raw_key = String::from(JWT_KEY);
        let value = String::from(VALUE);
        cache.insert(raw_key, value).await.unwrap();

        let raw_key = String::from(JWT_KEY);
        let result = cache.get(&raw_key).await.unwrap();
        assert_eq!(result, Some(String::from(VALUE)));
    }
}

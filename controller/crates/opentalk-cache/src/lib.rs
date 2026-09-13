// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use core::time::Duration;

pub use cache_error::CacheError;

mod cache_error;
pub mod hashing;
pub mod local;
pub mod overlay;
pub mod redis;

pub type Result<T, E = CacheError> = std::result::Result<T, E>;

#[async_trait::async_trait(?Send)]
pub trait CacheStorage<K, V> {
    fn ttl(&self) -> Duration;

    async fn get(&self, key: &K) -> Result<Option<V>>;

    /// Insert a value into the cache with the default TTL of the cache storage
    /// If the cache entry already exists, it will be updated with the new value and the TTL will be reset
    async fn insert(&self, key: K, value: V) -> Result<()>;

    /// Insert a value into the cache with customized TTL for this entry
    /// If the cache entry already exists, it will be updated with the new value and the TTL will be reset
    async fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) -> Result<()>;

    async fn invalidate(&self, key: &K) -> Result<()>;
}

#[derive(Debug, Clone, Copy)]
pub enum CacheUpdateMode {
    KeepTtl,
    ResetTtl,
}

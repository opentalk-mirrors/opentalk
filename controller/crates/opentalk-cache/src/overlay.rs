// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{marker::PhantomData, time::Duration};

use crate::{CacheStorage, Result};

pub struct Cache<K, V, BASE, OVERLAY>
where
    BASE: CacheStorage<K, V>,
    OVERLAY: CacheStorage<K, V>,
{
    base: BASE,
    overlay: OVERLAY,
    _phantom: PhantomData<(K, V)>,
}

impl<K, V, BASE, OVERLAY> Cache<K, V, BASE, OVERLAY>
where
    BASE: CacheStorage<K, V>,
    OVERLAY: CacheStorage<K, V>,
{
    pub fn new(base: BASE, overlay: OVERLAY) -> Self {
        Self {
            base,
            overlay,
            _phantom: PhantomData,
        }
    }
}

pub trait WithOverlay<K, V>: CacheStorage<K, V> + Sized {
    fn with_overlay<OVERLAY: CacheStorage<K, V>>(
        self,
        overlay: OVERLAY,
    ) -> Cache<K, V, Self, OVERLAY>;
}

impl<K, V, BASE: CacheStorage<K, V>> WithOverlay<K, V> for BASE {
    fn with_overlay<OVERLAY: CacheStorage<K, V>>(
        self,
        overlay: OVERLAY,
    ) -> Cache<K, V, Self, OVERLAY> {
        Cache::new(self, overlay)
    }
}

#[async_trait::async_trait(?Send)]
impl<K, V, BASE, OVERLAY> CacheStorage<K, V> for Cache<K, V, BASE, OVERLAY>
where
    K: Clone + 'static,
    V: Clone + 'static,
    BASE: CacheStorage<K, V>,
    OVERLAY: CacheStorage<K, V>,
{
    fn ttl(&self) -> Duration {
        std::cmp::min(self.base.ttl(), self.overlay.ttl())
    }

    async fn get(&self, key: &K) -> Result<Option<V>> {
        if let Ok(Some(value)) = self.overlay.get(key).await {
            return Ok(Some(value));
        }
        self.base.get(key).await
    }

    async fn insert(&self, key: K, value: V) -> Result<()> {
        self.base.insert(key.clone(), value.clone()).await?;
        self.overlay.insert(key, value).await
    }

    /// Insert an entry with a custom TTL
    ///
    /// Note that TTLs larger than the configured one will be ignored
    async fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) -> Result<()> {
        if ttl >= self.ttl() {
            return self.insert(key, value).await;
        }

        self.base
            .insert_with_ttl(key.clone(), value.clone(), ttl)
            .await?;
        self.overlay.insert_with_ttl(key, value, ttl).await
    }

    async fn invalidate(&self, key: &K) -> Result<()> {
        self.base.invalidate(key).await?;
        self.overlay.invalidate(key).await
    }
}

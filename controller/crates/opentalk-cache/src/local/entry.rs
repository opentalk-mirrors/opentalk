// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::{Duration, Instant};

use moka::Expiry;

use crate::CacheUpdateMode;

#[derive(Debug, Clone, Copy)]
pub(super) struct Entry<V> {
    value: V,
    ttl: Option<Duration>,
    mode: CacheUpdateMode,
}

impl<V> Entry<V> {
    pub(super) fn new(value: V, ttl: Duration, mode: CacheUpdateMode) -> Self {
        Self {
            value,
            ttl: Some(ttl),
            mode,
        }
    }

    pub(super) fn into_inner(self) -> V {
        self.value
    }
}

pub(super) struct EntryExpiry;

impl<K, V> Expiry<K, Entry<V>> for EntryExpiry {
    fn expire_after_create(
        &self,
        _key: &K,
        value: &Entry<V>,
        _created_at: Instant,
    ) -> Option<Duration> {
        value.ttl
    }

    fn expire_after_update(
        &self,
        _key: &K,
        value: &Entry<V>,
        _updated_at: Instant,
        duration_until_expiry: Option<Duration>,
    ) -> Option<Duration> {
        match value.mode {
            CacheUpdateMode::KeepTtl => duration_until_expiry,
            CacheUpdateMode::ResetTtl => value.ttl,
        }
    }
}

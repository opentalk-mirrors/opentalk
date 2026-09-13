// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::fmt::Display;

use redis::{ToRedisArgs, ToSingleRedisArg};

/// A Cache key with a [`ToRedisArgs`] implementation
///
/// Takes the prefix and cache key to turn them into a redis key
pub(super) struct RedisCacheKey<'a, K> {
    pub(super) prefix: &'a str,
    pub(super) key: &'a K,
}

impl<K: Display> Display for RedisCacheKey<'_, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "opentalk-cache:{}:{}", self.prefix, self.key)
    }
}

impl<D: Display> ToSingleRedisArg for RedisCacheKey<'_, D> {}

impl<D: Display> ToRedisArgs for RedisCacheKey<'_, D> {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        out.write_arg_fmt(self)
    }
}

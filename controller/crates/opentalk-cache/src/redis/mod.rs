// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod cache;
mod error;
mod key;
mod redis_cache_key;
mod value;

pub type Connection = redis::aio::ConnectionManager;

pub use cache::Cache;
pub use error::Error;
pub use key::Key;
use redis_cache_key::RedisCacheKey;
pub use value::Value;

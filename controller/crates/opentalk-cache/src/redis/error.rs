// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use snafu::Snafu;

use crate::CacheError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("Redis error: {}", source))]
    Redis { source: ::redis::RedisError },

    #[snafu(display("rkyv encode error: {source}"))]
    RkyvEncode { source: rkyv::rancor::Error },

    #[snafu(display("rkyv decode error: {source}"))]
    RkyvDecode { source: rkyv::rancor::Error },
}

impl From<Error> for CacheError {
    fn from(value: Error) -> Self {
        let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(value);
        CacheError::from(boxed)
    }
}

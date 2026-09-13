// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use rkyv::{
    Deserialize,
    api::high::{HighSerializer, HighValidator},
    bytecheck::CheckBytes,
    de::Pool,
    from_bytes,
    rancor::Strategy,
    ser::allocator::ArenaHandle,
    to_bytes,
    util::AlignedVec,
};
use snafu::ResultExt as _;

use crate::redis::error::{RkyvDecodeSnafu, RkyvEncodeSnafu};

pub trait Value: Sized {
    fn encode_for_redis(&self) -> Result<Vec<u8>, super::Error>;
    fn decode_from_redis(raw: &[u8]) -> Result<Self, super::Error>;
}

impl<T> Value for T
where
    T: for<'a> rkyv::Serialize<HighSerializer<AlignedVec, ArenaHandle<'a>, rkyv::rancor::Error>>
        + rkyv::Archive,
    T::Archived: for<'a> CheckBytes<HighValidator<'a, rkyv::rancor::Error>>
        + Deserialize<T, Strategy<Pool, rkyv::rancor::Error>>,
{
    fn encode_for_redis(&self) -> Result<Vec<u8>, super::Error> {
        to_bytes(self).context(RkyvEncodeSnafu).map(|v| v.to_vec())
    }

    fn decode_from_redis(raw: &[u8]) -> Result<Self, super::Error> {
        from_bytes(raw).context(RkyvDecodeSnafu)
    }
}

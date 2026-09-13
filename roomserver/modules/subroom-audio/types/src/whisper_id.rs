// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! The unique id for a whisper group
//!
//! The livekit room for the whisper group is named after the value of the whisper id.

use derive_more::{AsRef, Display, From, FromStr, Into};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique id of a whisper group inside a single room
#[derive(
    AsRef,
    Debug,
    Copy,
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Display,
    Into,
    From,
    FromStr,
    Serialize,
    Deserialize,
)]
pub struct WhisperId(Uuid);

impl WhisperId {
    /// Create a ZERO poll id, e.g. for testing purposes
    pub const fn nil() -> Self {
        Self(Uuid::nil())
    }

    /// Create a whisper id from a number for testing purposes
    pub const fn from_u128(id: u128) -> Self {
        Self(Uuid::from_u128(id))
    }

    /// Generate a new random whisper id
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

impl From<u128> for WhisperId {
    fn from(value: u128) -> Self {
        Self::from_u128(value)
    }
}

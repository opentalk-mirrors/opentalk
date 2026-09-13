// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use derive_more::{AsRef, Display, From, FromStr, Into};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
pub struct ConnectionId(Uuid);

impl ConnectionId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn nil() -> Self {
        Self(Uuid::nil())
    }

    pub fn from_u128(value: u128) -> Self {
        Self(Uuid::from_u128(value))
    }
}

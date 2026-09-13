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
pub struct DeviceId(Uuid);

impl DeviceId {
    pub fn nil() -> Self {
        DeviceId(Uuid::nil())
    }

    pub fn inner(&self) -> Uuid {
        self.0
    }

    pub fn from_u128(value: u128) -> Self {
        Self(Uuid::from_u128(value))
    }
}

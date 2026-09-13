// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Display, From, FromStr, Into};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ID of the message
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    AsRef,
    Display,
    From,
    FromStr,
    Into,
    Hash,
)]
pub struct MessageId(Uuid);

impl MessageId {
    /// Create a ZERO message id, e.g. for testing purposes
    pub const fn nil() -> Self {
        MessageId(Uuid::nil())
    }

    /// Create a message id from a number for testing purposes
    pub const fn from_u128(id: u128) -> Self {
        Self(Uuid::from_u128(id))
    }

    /// Generate a new random message id
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Display, From, FromStr, Into};
use uuid::Uuid;

use crate::utils::ExampleData;

/// The id of a breakout room
#[derive(
    AsRef, Display, From, FromStr, Into, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[cfg_attr(feature = "redis", derive(redis_args::ToRedisArgs))]
#[cfg_attr(feature = "redis", to_redis_args(fmt))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(BreakoutRoomId::example_data())))]
pub struct BreakoutRoomId(Uuid);

impl BreakoutRoomId {
    /// Create a ZERO breakout room id, e.g. for testing purposes
    pub const fn nil() -> Self {
        Self(Uuid::nil())
    }

    /// Create a breakout id from a number, e.g. for testing purposes
    pub const fn from_u128(id: u128) -> Self {
        Self(Uuid::from_u128(id))
    }

    /// Generate a new random breakout room id
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

impl ExampleData for BreakoutRoomId {
    fn example_data() -> Self {
        BreakoutRoomId::from_u128(0xBADCAB1E)
    }
}

// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! Module to use for (de-)serializing a [`std::time::Duration`] given in milliseconds.
use std::time::Duration;

use serde::{Deserialize, Deserializer, Serializer};

/// Deserialize function for the [`Duration`].
pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let ms: u64 = Deserialize::deserialize(deserializer)?;
    Ok(Duration::from_millis(ms))
}

/// Serialize function for the [`Duration`].
pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u128(duration.as_millis())
}

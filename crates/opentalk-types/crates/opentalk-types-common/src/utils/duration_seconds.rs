// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Module to use for (de-)serializing a [`std::time::Duration`] given in seconds.
use std::time::Duration;

use serde::{Deserialize, Deserializer, Serializer};

/// Deserialize function for the [`Duration`].
pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let seconds: u64 = Deserialize::deserialize(deserializer)?;
    Ok(Duration::from_secs(seconds))
}

/// Serialize function for the [`Duration`].
pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(duration.as_secs())
}

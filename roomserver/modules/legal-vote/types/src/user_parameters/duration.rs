// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use core::fmt;
use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

/// Minimum allowed length for a [`Duration`].
pub const MIN_DURATION_LENGTH: u32 = 5;

/// A validated duration with a minimum length constraint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u32")]
pub struct Duration(u32);

#[derive(Debug)]
pub struct TooShort {
    min_length: u32,
}

impl fmt::Display for TooShort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Duration must be at least {}.", self.min_length)
    }
}

impl TryFrom<u32> for Duration {
    type Error = TooShort;

    /// Converts a `u32` into a [`Duration`], enforcing the minimum length.
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value >= MIN_DURATION_LENGTH {
            Ok(Self(value))
        } else {
            Err(TooShort {
                min_length: MIN_DURATION_LENGTH,
            })
        }
    }
}

impl From<Duration> for std::time::Duration {
    fn from(duration: Duration) -> Self {
        Self::from_secs(duration.0 as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_length_min() {
        assert!(
            Duration::try_from(MIN_DURATION_LENGTH - 1).is_err(),
            "Duration should be at least {MIN_DURATION_LENGTH}."
        );

        assert!(
            Duration::try_from(MIN_DURATION_LENGTH).is_ok(),
            "Duration should be accepted if it is equal to {MIN_DURATION_LENGTH}."
        );
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialization() {
        let produced = serde_json::to_value(Duration::try_from(10).unwrap()).unwrap();
        let expected = json!(10);

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization() {
        let produced: Duration = serde_json::from_value(json!(10)).unwrap();
        let expected = Duration::try_from(10).unwrap();
        assert_eq!(produced, expected);

        let produced: Result<Duration, _> = serde_json::from_value(json!(4));
        assert!(produced.is_err());
    }
}

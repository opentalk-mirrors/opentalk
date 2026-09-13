// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

use crate::too_long_error::TooLongError;

/// Maximum allowed length for a [`Topic`].
pub const MAX_TOPIC_LENGTH: usize = 500;

/// A validated topic string with a maximum length constraint.
#[derive(Debug, Clone, PartialEq, Eq, derive_more::Display, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct Topic(String);

fn ensure_is_valid(s: &str) -> Result<(), TooLongError> {
    if s.len() <= MAX_TOPIC_LENGTH {
        Ok(())
    } else {
        Err(TooLongError {
            max_length: MAX_TOPIC_LENGTH,
        })
    }
}

impl std::str::FromStr for Topic {
    type Err = TooLongError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure_is_valid(s)?;
        Ok(Self(s.to_string()))
    }
}

impl TryFrom<String> for Topic {
    type Error = TooLongError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ensure_is_valid(&value)?;
        Ok(Self(value))
    }
}

impl TryFrom<&str> for Topic {
    type Error = TooLongError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{self, json};

    use super::*;

    #[test]
    fn topic_max_length() {
        assert!(
            Topic::try_from("a".repeat(MAX_TOPIC_LENGTH + 1)).is_err(),
            "Topic should be rejected if it exceeds {MAX_TOPIC_LENGTH} characters"
        );

        assert!(
            Topic::try_from("a".repeat(MAX_TOPIC_LENGTH)).is_ok(),
            "Topic should be accepted if it is within the limit"
        );
    }

    #[test]
    fn serialize_topic() {
        let produced = serde_json::to_value(Topic::try_from("Test Topic").unwrap()).unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        insta::assert_snapshot!(raw, @r#""Test Topic""#);
    }

    #[test]
    fn deserialize_topic() {
        let produced: Topic = serde_json::from_value(json!("Test Topic")).unwrap();
        let expected = Topic::try_from("Test Topic").unwrap();

        assert_eq!(produced, expected);

        let produced: Result<Topic, _> =
            serde_json::from_value(json!("a".repeat(MAX_TOPIC_LENGTH + 1)));

        assert!(produced.is_err());
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

use crate::too_long_error::TooLongError;

/// Maximum allowed length for a [`Name`].
pub const MAX_NAME_LENGTH: usize = 150;

/// A validated name string with a maximum length constraint.
#[derive(Debug, Clone, PartialEq, Eq, derive_more::Display, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct Name(String);

fn ensure_is_valid(s: &str) -> Result<(), TooLongError> {
    if s.len() <= MAX_NAME_LENGTH {
        Ok(())
    } else {
        Err(TooLongError {
            max_length: MAX_NAME_LENGTH,
        })
    }
}

impl std::str::FromStr for Name {
    type Err = TooLongError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure_is_valid(s)?;
        Ok(Self(s.to_string()))
    }
}

impl TryFrom<String> for Name {
    type Error = TooLongError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ensure_is_valid(&value)?;
        Ok(Self(value))
    }
}

impl TryFrom<&str> for Name {
    type Error = TooLongError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn name_max_length() {
        assert!(
            Name::try_from("a".repeat(MAX_NAME_LENGTH + 1)).is_err(),
            "Name should be rejected if it exceeds {MAX_NAME_LENGTH} characters"
        );

        assert!(
            Name::try_from("a".repeat(MAX_NAME_LENGTH)).is_ok(),
            "Name should be accepted if it is within the limit"
        );
    }

    #[test]
    fn serialize_name() {
        let produced = serde_json::to_value(Name::try_from("Test Name").unwrap()).unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#""Test Name""#);
    }

    #[test]
    fn deserialize_name() {
        let produced: Name = serde_json::from_value(json!("Test Name")).unwrap();
        let expected = Name::try_from("Test Name").unwrap();

        assert_eq!(produced, expected);

        let produced: Result<Name, _> =
            serde_json::from_value(json!("a".repeat(MAX_NAME_LENGTH + 1)));

        assert!(produced.is_err());
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

use crate::too_long_error::TooLongError;

/// Maximum allowed length for a [`Subtitle`].
pub const MAX_SUBTITLE_LENGTH: usize = 255;

/// A validated subtitle string with a maximum length constraint.
#[derive(Debug, Clone, PartialEq, Eq, derive_more::Display, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct Subtitle(String);

fn ensure_is_valid(s: &str) -> Result<(), TooLongError> {
    if s.len() <= MAX_SUBTITLE_LENGTH {
        Ok(())
    } else {
        Err(TooLongError {
            max_length: MAX_SUBTITLE_LENGTH,
        })
    }
}

impl std::str::FromStr for Subtitle {
    type Err = TooLongError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure_is_valid(s)?;
        Ok(Self(s.to_string()))
    }
}

impl TryFrom<String> for Subtitle {
    type Error = TooLongError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ensure_is_valid(&value)?;
        Ok(Self(value))
    }
}

impl TryFrom<&str> for Subtitle {
    type Error = TooLongError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[cfg(test)]
mod test {
    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
    struct TestStruct {
        subtitle: Subtitle,
    }

    #[test]
    fn serialize_subtitle() {
        let produced = serde_json::to_value(TestStruct {
            subtitle: Subtitle::try_from("Test Subtitle").unwrap(),
        })
        .unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "subtitle": "Test Subtitle"
        }
        "#);
    }

    #[test]
    fn deserialize_subtitle() {
        let produced: Subtitle = serde_json::from_value(json!("Test Subtitle")).unwrap();
        let expected = Subtitle::try_from("Test Subtitle").unwrap();

        assert_eq!(produced, expected);

        let produced: Result<Subtitle, _> =
            serde_json::from_value(json!("a".repeat(MAX_SUBTITLE_LENGTH + 1)));

        assert!(produced.is_err());
    }

    #[test]
    fn subtitle_max_length() {
        assert!(
            Subtitle::try_from("a".repeat(MAX_SUBTITLE_LENGTH + 1)).is_err(),
            "Subtitle should be rejected if it exceeds {MAX_SUBTITLE_LENGTH} characters."
        );

        assert!(
            Subtitle::try_from("a".repeat(MAX_SUBTITLE_LENGTH)).is_ok(),
            "Subtitle should be accepted if it is within the limit."
        );
    }
}

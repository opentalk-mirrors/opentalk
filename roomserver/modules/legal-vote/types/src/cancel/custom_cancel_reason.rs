// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling custom cancel reason for the `legal-vote` namespace.

use serde::{Deserialize, Serialize};

use crate::too_long_error::TooLongError;

/// The maximum allowed length for a custom cancel reason.
pub const MAX_CUSTOM_CANCEL_REASON_LENGTH: usize = 255;

/// A custom reason for canceling a vote.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, derive_more::Display, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct CustomCancelReason(String);

fn ensure_is_valid(s: &str) -> Result<(), TooLongError> {
    if s.len() <= MAX_CUSTOM_CANCEL_REASON_LENGTH {
        Ok(())
    } else {
        Err(TooLongError {
            max_length: MAX_CUSTOM_CANCEL_REASON_LENGTH,
        })
    }
}

impl std::str::FromStr for CustomCancelReason {
    type Err = TooLongError;

    fn from_str(s: &str) -> Result<Self, TooLongError> {
        ensure_is_valid(s)?;
        Ok(Self(s.to_string()))
    }
}

impl TryFrom<String> for CustomCancelReason {
    type Error = TooLongError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ensure_is_valid(&value)?;
        Ok(Self(value))
    }
}

impl TryFrom<&str> for CustomCancelReason {
    type Error = TooLongError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancel_reason_max_length() {
        assert!(
            CustomCancelReason::try_from("a".repeat(MAX_CUSTOM_CANCEL_REASON_LENGTH)).is_ok(),
            "The CustomCancelReason should be allowed to be {MAX_CUSTOM_CANCEL_REASON_LENGTH} chars long."
        );

        assert!(
            CustomCancelReason::try_from("a".repeat(MAX_CUSTOM_CANCEL_REASON_LENGTH + 1)).is_err(),
            "The CustomCancelReason should not be allowed to be longer than {MAX_CUSTOM_CANCEL_REASON_LENGTH}."
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
        let produced =
            serde_json::to_value(CustomCancelReason::try_from("Test Reason").unwrap()).unwrap();
        let expected = json!("Test Reason");
        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization() {
        let produced: CustomCancelReason = serde_json::from_value(json!("Test Reason")).unwrap();
        let expected = CustomCancelReason::try_from("Test Reason").unwrap();
        assert_eq!(produced, expected);

        let produced: Result<CustomCancelReason, _> =
            serde_json::from_value(json!("a".repeat(MAX_CUSTOM_CANCEL_REASON_LENGTH + 1)));
        assert!(produced.is_err());
    }
}

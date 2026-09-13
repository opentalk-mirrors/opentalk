// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use derive_more::Display;
use snafu::Snafu;

use crate::utils::ExampleData;

/// A room suffix that can be appended to a [`RoomName`](super::RoomName) to secure it against brute-force attacks.
#[derive(Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "diesel",
    derive(
        opentalk_diesel_newtype::DieselNewtype,
        diesel::expression::AsExpression,
        diesel::deserialize::FromSqlRow
    )
)]
#[cfg_attr(feature="diesel",
    diesel(sql_type = diesel::sql_types::VarChar),
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(RoomSuffix::example_data())))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RoomSuffix(String);

impl RoomSuffix {
    /// Generates a new random [`RoomSuffix`].
    #[cfg(feature = "rand")]
    pub fn generate(length: u8) -> Self {
        use rand::seq::IndexedRandom;

        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

        let mut rng = rand::rng();
        let code = (0..length)
            .map(|_| *CHARSET.choose(&mut rng).expect("charset is never empty") as char)
            .collect();

        Self(code)
    }

    /// Create a ZERO room suffix, e.g. for testing purposes.
    pub fn nil(length: u8) -> Self {
        Self("0".repeat(length as usize))
    }

    /// The number of characters in the room suffix.
    pub fn char_count(&self) -> u8 {
        self.0.chars().count() as u8
    }

    /// Returns the room suffix as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the [`RoomSuffix`], returning the inner [`String`].
    pub fn into_inner(self) -> String {
        self.0
    }
}

/// Errors that can occur when trying to convert a [`String`] into a [`RoomSuffix`].
#[derive(Debug, Snafu, PartialEq, Eq, Clone, Copy)]
#[snafu(display("Room suffix contains invalid characters"))]
pub struct InvalidCharacters;

impl TryFrom<String> for RoomSuffix {
    type Error = InvalidCharacters;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.chars().all(|c| matches!(c, 'a'..='z' | '0'..='9')) {
            Ok(Self(value))
        } else {
            Err(InvalidCharacters)
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for RoomSuffix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;

        String::deserialize(deserializer)?
            .try_into()
            .map_err(D::Error::custom)
    }
}

impl ExampleData for RoomSuffix {
    fn example_data() -> Self {
        const EXAMPLE_ROOM_SUFFIX_LENGTH: u8 = 16;

        Self::nil(EXAMPLE_ROOM_SUFFIX_LENGTH)
    }
}

#[cfg(test)]
mod tests {
    use super::{InvalidCharacters, RoomSuffix};

    #[test]
    fn invalid_character_is_rejected() {
        let result = RoomSuffix::try_from("abc!".to_string());
        assert_eq!(result, Err(InvalidCharacters));
    }

    #[test]
    fn upper_case_is_rejected() {
        let result = RoomSuffix::try_from("ABC".to_string());
        assert_eq!(result, Err(InvalidCharacters));
    }

    #[test]
    fn valid_suffix_is_accepted() {
        let value = "abc123".to_string();
        let result = RoomSuffix::try_from(value.clone());
        assert_eq!(result.map(RoomSuffix::into_inner), Ok(value));
    }
}

#[cfg(all(test, feature = "rand"))]
mod rand_tests {
    use super::RoomSuffix;

    #[test]
    fn generate_creates_suffix_with_correct_length() {
        let suffix = RoomSuffix::generate(10);
        assert_eq!(suffix.char_count(), 10);
    }

    #[test]
    fn generate_uses_only_valid_characters() {
        let suffix = RoomSuffix::generate(100);
        assert!(
            suffix
                .as_str()
                .chars()
                .all(|c| matches!(c, 'a'..='z' | '0'..='9'))
        );
    }

    #[test]
    fn generate_produces_different_results() {
        let a = RoomSuffix::generate(32);
        let b = RoomSuffix::generate(32);
        assert_ne!(a, b);
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::RoomSuffix;
    use crate::utils::ExampleData as _;

    #[test]
    fn serialize() {
        assert_eq!(
            serde_json::to_value(RoomSuffix::example_data()).unwrap(),
            json!("0000000000000000")
        );
    }

    #[test]
    fn deserialize() {
        assert_eq!(
            serde_json::from_value::<RoomSuffix>(json!("0000000000000000")).unwrap(),
            RoomSuffix::example_data()
        );
    }

    #[test]
    fn deserialize_rejects_invalid_characters() {
        assert!(serde_json::from_value::<RoomSuffix>(json!("ABC!")).is_err());
    }
}

// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use derive_more::Display;
use snafu::Snafu;
use uuid::Uuid;

use crate::utils::ExampleData;

const MIN_ROOM_NAME_LENGTH: usize = 3;
const MAX_ROOM_NAME_LENGTH: usize = 40;
const ALLOWED_NON_ALPHANUMERIC_CHARS: &[char] = &['.', '-'];

/// A valid name for a room.
///
/// A room name must
///
/// * be at least 3 characters long
/// * be at most 40 characters long
/// * contain only `a-z`, `0-9`, `.` and `-`
/// * not be a valid UUID
///
/// The "not a valid UUID" rule exists so that a [`RoomName`] can never be confused with a
/// [`RoomId`](super::RoomId): both can appear in the same URL position and are resolved via the untagged
/// [`RoomIdOrAlias`](super::RoomIdOrAlias) enum. Without it, users could hijack rooms by creating a name that
/// collides with an existing room's ID.
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
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(RoomName::example_data())))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RoomName(String);

impl RoomName {
    /// Returns the room name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the [`RoomName`], returning the inner [`String`].
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl ExampleData for RoomName {
    fn example_data() -> Self {
        Self("personal-room-name".to_owned())
    }
}

/// An error that can occur when trying to convert a [`String`] into a [`RoomName`].
#[derive(Debug, Snafu, PartialEq, Eq, Clone, Copy)]
pub enum TryFromRoomNameError {
    /// The room name is too short
    #[snafu(display("Room name must have at least {MIN_ROOM_NAME_LENGTH} characters"))]
    TooShort,
    /// The room name is too long
    #[snafu(display("Room name must have at most {MAX_ROOM_NAME_LENGTH} characters"))]
    TooLong,
    /// The room name contains invalid characters
    #[snafu(display("Room name contains invalid characters"))]
    InvalidCharacter,
    /// The room name is a valid UUID
    #[snafu(display("Room name must not be a valid UUID"))]
    UuidNotAllowed,
}

impl TryFrom<String> for RoomName {
    type Error = TryFromRoomNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let count = value.chars().count();

        if count < MIN_ROOM_NAME_LENGTH {
            return Err(TryFromRoomNameError::TooShort);
        }

        if count > MAX_ROOM_NAME_LENGTH {
            return Err(TryFromRoomNameError::TooLong);
        }

        if !value.chars().all(|char| {
            matches!(char, 'a'..='z' | '0'..='9') || ALLOWED_NON_ALPHANUMERIC_CHARS.contains(&char)
        }) {
            return Err(TryFromRoomNameError::InvalidCharacter);
        }

        // Reject UUID-shaped names so that a `RoomName` can never collide with a `RoomId`. Without this, a name that
        // parses as a UUID would deserialize as a `RoomId` when used in a `RoomIdOrAlias` context, which would allow
        // users to hijack rooms by creating a name that collides with an existing room's ID.
        if Uuid::parse_str(&value).is_ok() {
            return Err(TryFromRoomNameError::UuidNotAllowed);
        }

        Ok(RoomName(value))
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for RoomName {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_is_rejected() {
        let result = RoomName::try_from("ab".to_string());
        assert_eq!(result, Err(TryFromRoomNameError::TooShort));
    }

    #[test]
    fn minimum_length_is_accepted() {
        let name = "a".repeat(MIN_ROOM_NAME_LENGTH);
        let result = RoomName::try_from(name.clone());
        assert_eq!(result, Ok(RoomName(name)));
    }

    #[test]
    fn too_long_is_rejected() {
        let name = "a".repeat(MAX_ROOM_NAME_LENGTH + 1);
        let result = RoomName::try_from(name);
        assert_eq!(result, Err(TryFromRoomNameError::TooLong));
    }

    #[test]
    fn maximum_length_is_accepted() {
        let name = "a".repeat(MAX_ROOM_NAME_LENGTH);
        let result = RoomName::try_from(name.clone());
        assert_eq!(result, Ok(RoomName(name)));
    }

    #[test]
    fn all_allowed_character_classes_are_accepted() {
        let name = "abc-0.9-xyz".to_string();
        let result = RoomName::try_from(name.clone());
        assert_eq!(result, Ok(RoomName(name)));
    }

    #[test]
    fn non_ascii_is_rejected() {
        let result = RoomName::try_from("👋👋👋".to_string());
        assert_eq!(result, Err(TryFromRoomNameError::InvalidCharacter));
    }

    #[test]
    fn upper_case_is_rejected() {
        let result = RoomName::try_from("Room-name".to_string());
        assert_eq!(result, Err(TryFromRoomNameError::InvalidCharacter));
    }

    #[test]
    fn invalid_character_is_rejected() {
        let result = RoomName::try_from("room/name".to_string());
        assert_eq!(result, Err(TryFromRoomNameError::InvalidCharacter));
    }

    #[test]
    fn uuid_is_rejected() {
        let value = Uuid::nil().to_string();
        let result = RoomName::try_from(value);
        assert_eq!(result, Err(TryFromRoomNameError::UuidNotAllowed));
    }

    #[test]
    fn example_data_is_valid() {
        let example = RoomName::example_data();
        let result = RoomName::try_from(example.clone().into_inner());
        assert_eq!(result, Ok(example));
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::RoomName;
    use crate::utils::ExampleData as _;

    #[test]
    fn serialize() {
        assert_eq!(
            serde_json::to_value(RoomName::example_data()).unwrap(),
            json!("personal-room-name")
        );
    }

    #[test]
    fn deserialize() {
        assert_eq!(
            serde_json::from_value::<RoomName>(json!("personal-room-name")).unwrap(),
            RoomName::example_data()
        );
    }

    #[test]
    fn deserialize_rejects_invalid_name() {
        assert!(serde_json::from_value::<RoomName>(json!("Invalid Name")).is_err());
    }
}

// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::fmt::Display;

use crate::{
    rooms::{RoomName, RoomSuffix},
    utils::ExampleData,
};

/// A human readable alias for a room.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoomAlias {
    /// The user defined name of the room.
    pub name: RoomName,
    /// An optional room suffix that is appended to the room name to secure it against brute-force attacks.
    pub suffix: Option<RoomSuffix>,
}

impl Display for RoomAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(suffix) = &self.suffix {
            write!(f, "{}_{}", self.name, suffix)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for RoomAlias {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for RoomAlias {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;

        let value = String::deserialize(deserializer)?;
        let (name, suffix) = match value.split_once('_') {
            Some((name, suffix)) => (name.to_owned(), Some(suffix.to_owned())),
            None => (value, None),
        };

        let name = name.try_into().map_err(D::Error::custom)?;
        let suffix = suffix
            .map(|suffix| suffix.try_into())
            .transpose()
            .map_err(D::Error::custom)?;

        Ok(RoomAlias { name, suffix })
    }
}

// `RoomAlias` serializes as a string (see its `Serialize`/`Display` impls), so the derived object schema would be wrong.
#[cfg(feature = "utoipa")]
mod impl_utoipa {
    use serde_json::json;
    use utoipa::{
        PartialSchema, ToSchema,
        openapi::{ObjectBuilder, RefOr, Schema, Type},
    };

    use super::RoomAlias;
    use crate::utils::ExampleData;

    impl PartialSchema for RoomAlias {
        fn schema() -> RefOr<Schema> {
            ObjectBuilder::new()
                .schema_type(Type::String)
                .description(Some("A human readable alias for a room."))
                .examples([json!(RoomAlias::example_data())])
                .into()
        }
    }

    impl ToSchema for RoomAlias {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

impl ExampleData for RoomAlias {
    fn example_data() -> Self {
        Self {
            name: RoomName::example_data(),
            suffix: Some(RoomSuffix::example_data()),
        }
    }
}

/// Extension trait providing a way to destructure an optional [`RoomAlias`] into its optional name and suffix components.
pub trait OptionalRoomAliasExt {
    /// Split an optional [`RoomAlias`] into its optional [`RoomName`] and [`RoomSuffix`].
    fn into_parts(self) -> (Option<RoomName>, Option<RoomSuffix>);
}

impl OptionalRoomAliasExt for Option<RoomAlias> {
    fn into_parts(self) -> (Option<RoomName>, Option<RoomSuffix>) {
        match self {
            Some(RoomAlias { name, suffix }) => (Some(name), suffix),
            None => (None, None),
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_with_suffix() {
        let expected = json!("personal-room-name_0000000000000000");
        let produced = serde_json::to_value(RoomAlias::example_data()).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn deserialize_with_suffix() {
        let expected = RoomAlias::example_data();
        let produced =
            serde_json::from_value::<RoomAlias>(json!("personal-room-name_0000000000000000"))
                .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_without_suffix() {
        let expected = json!("personal-room-name");
        let produced = serde_json::to_value(RoomAlias {
            name: RoomName::example_data(),
            suffix: None,
        })
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn deserialize_without_suffix() {
        let expected = RoomAlias {
            name: RoomName::example_data(),
            suffix: None,
        };
        let produced = serde_json::from_value(json!("personal-room-name")).unwrap();

        assert_eq!(expected, produced);
    }
}

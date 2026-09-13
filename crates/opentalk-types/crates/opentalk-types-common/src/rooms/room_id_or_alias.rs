// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use derive_more::Display;

use crate::{
    rooms::{RoomAlias, RoomId},
    utils::ExampleData,
};

/// An enum that represents either a room id or a room alias.
#[derive(Debug, Display, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(RoomIdOrAlias::example_data())))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(untagged)
)]
pub enum RoomIdOrAlias {
    /// A room id.
    Id(RoomId),
    /// A room alias.
    Alias(RoomAlias),
}

impl ExampleData for RoomIdOrAlias {
    fn example_data() -> Self {
        Self::Id(RoomId::example_data())
    }
}

impl From<RoomId> for RoomIdOrAlias {
    fn from(value: RoomId) -> Self {
        Self::Id(value)
    }
}

impl From<RoomAlias> for RoomIdOrAlias {
    fn from(value: RoomAlias) -> Self {
        Self::Alias(value)
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::RoomIdOrAlias;
    use crate::{
        rooms::{RoomAlias, RoomId, RoomName, RoomSuffix},
        utils::ExampleData as _,
    };

    #[test]
    fn serialize_id() {
        assert_eq!(
            serde_json::to_value(RoomIdOrAlias::Id(RoomId::example_data())).unwrap(),
            json!("00000000-0000-0000-0000-0000abadcafe")
        );
    }

    #[test]
    fn deserialize_id() {
        let expected = RoomIdOrAlias::Id(RoomId::example_data());
        let produced =
            serde_json::from_value(json!("00000000-0000-0000-0000-0000abadcafe")).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_alias_with_suffix() {
        assert_eq!(
            serde_json::to_value(RoomIdOrAlias::Alias(RoomAlias {
                name: RoomName::example_data(),
                suffix: Some(RoomSuffix::example_data()),
            }))
            .unwrap(),
            json!("personal-room-name_0000000000000000")
        );
    }

    #[test]
    fn deserialize_alias_with_suffix() {
        let expected = RoomIdOrAlias::Alias(RoomAlias::example_data());
        let produced =
            serde_json::from_value(json!("personal-room-name_0000000000000000")).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_alias_without_suffix() {
        assert_eq!(
            serde_json::to_value(RoomIdOrAlias::Alias(RoomAlias {
                name: RoomName::example_data(),
                suffix: None,
            }))
            .unwrap(),
            json!("personal-room-name")
        );
    }

    #[test]
    fn deserialize_alias_without_suffix() {
        let expected = RoomIdOrAlias::Alias(RoomAlias {
            name: RoomName::example_data(),
            suffix: None,
        });
        let produced = serde_json::from_value(json!("personal-room-name")).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn deserialize_invalid_value_is_rejected() {
        assert!(serde_json::from_value::<RoomIdOrAlias>(json!("Invalid Value")).is_err());
    }
}

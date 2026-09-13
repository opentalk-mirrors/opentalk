// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::utils::ExampleData;

/// The quota types that can be enforced on tenants.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, strum::Display, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(
    feature = "clap",
    derive(clap::ValueEnum),
    clap(rename_all = "snake_case")
)]
#[cfg_attr(
    feature = "typescript",
    derive(ts_rs::TS),
    ts(export_to = "common/", repr(enum = name))
)]
pub enum QuotaType {
    /// This quota limits the total amount of data, measured bytes, that can be
    /// stored by the tenant. This is a soft limit which allows the user to store
    /// files as long as their usage is below the limit. Once the limit is reached
    /// or exceeded, no new data can be stored.
    MaxStorage,

    /// This quota restricts the total duration for which a tenant can utilize a
    /// meeting room, measured in seconds.
    RoomTimeLimitSecs,

    /// This quota sets a limit on the number of participants that can join a room.
    RoomParticipantLimit,
}

#[cfg(feature = "utoipa")]
mod impl_utoipa {
    //! The `#[derive(utoipa::ToSchema)] implementation does not yet properly support
    //! exposing schema information of types wrapped by the NewType pattern, therefore
    //! a manual implementation is required for now.
    //! Issue: <https://github.com/juhaku/utoipa/issues/663>

    use serde_json::json;
    use utoipa::{
        PartialSchema, ToSchema,
        openapi::{ObjectBuilder, RefOr, Schema, Type, schema::AnyOf},
    };

    use super::QuotaType;
    use crate::utils::ExampleData as _;

    impl PartialSchema for QuotaType {
        fn schema() -> RefOr<Schema> {
            Schema::AnyOf(AnyOf {
                items: vec![
                    ObjectBuilder::new()
                        .schema_type(Type::String)
                        .description(Some("A quota type"))
                        .examples([json!(QuotaType::example_data())])
                        .into(),
                ],
                description: None,
                example: Some(json!(Self::example_data())),
                discriminator: None,
                ..Default::default()
            })
            .into()
        }
    }

    impl ToSchema for QuotaType {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

impl ExampleData for QuotaType {
    fn example_data() -> Self {
        Self::MaxStorage
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn quota_type_json() {
        use std::collections::BTreeMap;

        use serde_json::json;

        let quota = BTreeMap::from([
            (QuotaType::MaxStorage, 11u64),
            (QuotaType::RoomTimeLimitSecs, 12u64),
            (QuotaType::RoomParticipantLimit, 13u64),
        ]);
        let quota_json_repr =
            serde_json::to_value(quota.clone()).expect("QuotaType must be serializable");

        assert_eq!(
            quota_json_repr,
            json!({
                "max_storage": 11,
                "room_time_limit_secs": 12,
                "room_participant_limit": 13,
            })
        );
        assert_eq!(
            quota,
            serde_json::from_value(quota_json_repr).expect("Must be deserialize")
        );
    }
}

#[cfg(all(test, feature = "clap"))]
mod clap_tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn quota_type_string() {
        use std::str::FromStr;

        assert_eq!(
            QuotaType::from_str("max_storage").unwrap(),
            QuotaType::MaxStorage
        );
        assert_eq!(
            QuotaType::from_str("room_time_limit_secs").unwrap(),
            QuotaType::RoomTimeLimitSecs
        );
        assert_eq!(
            QuotaType::from_str("room_participant_limit").unwrap(),
            QuotaType::RoomParticipantLimit
        );
    }
}

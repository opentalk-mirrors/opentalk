// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{fmt::Display, ops::Add, str::FromStr};

use chrono::{DateTime, NaiveDateTime, TimeZone as _, Utc};
use opentalk_types_common::{time::Timestamp, utils::ExampleData};

use crate::events::UTC_DT_FORMAT;

/// ID of an EventInstance
///
/// Is created from the starts_at datetime of the original recurrence (original meaning that exceptions don't change
/// the instance id).
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::From,
    derive_more::Into,
    derive_more::AsRef,
)]
pub struct InstanceId(Timestamp);

impl From<DateTime<Utc>> for InstanceId {
    fn from(dt: DateTime<Utc>) -> Self {
        InstanceId(dt.into())
    }
}

impl From<InstanceId> for DateTime<Utc> {
    fn from(id: InstanceId) -> Self {
        id.0.into()
    }
}

impl Display for InstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format(UTC_DT_FORMAT))
    }
}

impl FromStr for InstanceId {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dt: DateTime<Utc> = NaiveDateTime::parse_from_str(s, UTC_DT_FORMAT)?.and_utc();
        Ok(Self(dt.into()))
    }
}

impl Add<chrono::Duration> for InstanceId {
    type Output = Self;

    fn add(self, rhs: chrono::Duration) -> Self::Output {
        InstanceId(self.0 + rhs)
    }
}

impl ExampleData for InstanceId {
    fn example_data() -> Self {
        Self(Utc.with_ymd_and_hms(2024, 7, 5, 17, 2, 42).unwrap().into())
    }
}

#[cfg(feature = "utoipa")]
mod impl_utoipa {
    use serde_json::json;
    use utoipa::{
        PartialSchema, ToSchema,
        openapi::{ObjectBuilder, RefOr, Schema, Type},
    };

    use super::InstanceId;

    impl PartialSchema for InstanceId {
        fn schema() -> RefOr<Schema> {
            ObjectBuilder::new()
                .schema_type(Type::String)
                .description(Some("An event instance id"))
                .examples([json!("20240720T152342Z")])
                .into()
        }
    }

    impl ToSchema for InstanceId {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impls {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{InstanceId, UTC_DT_FORMAT};

    const DT_FORMAT: &str = "%Y%m%dT%H%M%S%z";
    struct InstanceIdVisitor;

    impl serde::de::Visitor<'_> for InstanceIdVisitor {
        type Value = InstanceId;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(
                formatter,
                "timestamp in '{DT_FORMAT}' or '{UTC_DT_FORMAT}' format"
            )
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let utc_date_time = DateTime::parse_from_str(v, DT_FORMAT)
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|_| {
                    NaiveDateTime::parse_from_str(v, UTC_DT_FORMAT).map(|ndt| ndt.and_utc())
                })
                .map_err(|_| {
                    serde::de::Error::invalid_value(serde::de::Unexpected::Str(v), &self)
                })?;

            Ok(InstanceId(utc_date_time.into()))
        }
    }

    impl<'de> Deserialize<'de> for InstanceId {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_str(InstanceIdVisitor)
        }
    }

    impl Serialize for InstanceId {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.0
                .format(UTC_DT_FORMAT)
                .to_string()
                .serialize(serializer)
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use std::time::UNIX_EPOCH;

    use serde_json::json;

    use super::InstanceId;

    #[test]
    fn serialize_utc() {
        let json = json!("19700101T000000Z");

        let instance_id: InstanceId = serde_json::from_value(json).unwrap();

        assert_eq!(instance_id.0, UNIX_EPOCH.into())
    }

    #[test]
    fn serialize_utc_plus_one() {
        let json = json!("19700101T010000+0100");

        let instance_id: InstanceId = serde_json::from_value(json).unwrap();

        assert_eq!(instance_id.0, UNIX_EPOCH.into())
    }
}

#[cfg(test)]
mod tests {
    use std::time::UNIX_EPOCH;

    use crate::events::InstanceId;

    #[test]
    fn from_str() {
        let s = "19700101T000000Z";

        let instance_id: InstanceId = s.parse().unwrap();

        assert_eq!(instance_id.0, UNIX_EPOCH.into())
    }
}

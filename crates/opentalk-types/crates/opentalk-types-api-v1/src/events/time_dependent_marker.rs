// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Marker type that serializes to `false`.
///
/// This ensures `is_time_independent: false` appears in JSON for TimeDependent variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TimeDependentMarker;

#[cfg(feature = "serde")]
impl serde::Serialize for TimeDependentMarker {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bool(false)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for TimeDependentMarker {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = bool::deserialize(deserializer)?;
        if !value {
            Ok(TimeDependentMarker)
        } else {
            Err(serde::de::Error::custom(
                "Expected is_time_independent to be false for TimeDependent variant.",
            ))
        }
    }
}

#[cfg(feature = "utoipa")]
mod impl_utoipa {
    use serde_json::json;
    use utoipa::{
        PartialSchema, ToSchema,
        openapi::{ObjectBuilder, RefOr, Type, schema::Schema},
    };

    use crate::events::TimeDependentMarker;

    impl PartialSchema for TimeDependentMarker {
        fn schema() -> RefOr<Schema> {
            ObjectBuilder::new()
                .schema_type(Type::Boolean)
                .enum_values(Some([json!(false)]))
                .description(Some("Marker type that serializes to `false`."))
                .examples([json!(false)])
                .into()
        }
    }

    impl ToSchema for TimeDependentMarker {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

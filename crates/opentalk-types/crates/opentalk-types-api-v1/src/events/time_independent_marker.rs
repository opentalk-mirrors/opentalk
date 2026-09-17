// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Marker type that serializes to `true`
///
/// This ensures `is_time_independent: true` appears in JSON for TimeIndependent variant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TimeIndependentMarker;

#[cfg(feature = "serde")]
impl serde::Serialize for TimeIndependentMarker {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bool(true)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for TimeIndependentMarker {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = bool::deserialize(deserializer)?;
        if value {
            Ok(TimeIndependentMarker)
        } else {
            Err(serde::de::Error::custom(
                "Expected is_time_independent to be true for TimeIndependent variant.",
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

    use crate::events::TimeIndependentMarker;

    impl PartialSchema for TimeIndependentMarker {
        fn schema() -> RefOr<Schema> {
            ObjectBuilder::new()
                .schema_type(Type::Boolean)
                .enum_values(Some([json!(true)]))
                .description(Some("Marker type that serializes to `true`."))
                .examples([json!(true)])
                .into()
        }
    }

    impl ToSchema for TimeIndependentMarker {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

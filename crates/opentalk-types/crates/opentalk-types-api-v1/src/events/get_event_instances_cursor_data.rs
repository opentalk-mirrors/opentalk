// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{pagination::Page, utils::ExampleData};

/// Data stored inside the `GET /events/{event_id}/instances` query cursor
#[derive(Default, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GetEventInstancesCursorData {
    /// Page number
    #[cfg_attr(feature = "serde", serde(default))]
    pub page: Page,
}

impl ExampleData for GetEventInstancesCursorData {
    fn example_data() -> Self {
        Self {
            page: 4i64.try_into().unwrap(),
        }
    }
}

#[cfg(feature = "utoipa")]
mod impl_utoipa {
    use opentalk_types_common::utils::ExampleData as _;
    use serde_json::json;
    use utoipa::{
        PartialSchema, ToSchema,
        openapi::{ObjectBuilder, RefOr, Schema, Type},
    };

    use super::GetEventInstancesCursorData;
    use crate::pagination::Cursor;

    impl PartialSchema for GetEventInstancesCursorData {
        fn schema() -> RefOr<Schema> {
            ObjectBuilder::new()
                .schema_type(Type::String)
                .description(Some("A cursor pointing to an event instance"))
                .examples([json!(Cursor(Self::example_data()))])
                .into()
        }
    }

    impl ToSchema for GetEventInstancesCursorData {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use opentalk_types_common::utils::ExampleData as _;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::GetEventInstancesCursorData;

    #[test]
    fn serialize_default() {
        let example = GetEventInstancesCursorData::default();
        assert_eq!(json!(example), json!({"page": 1}));
    }

    #[test]
    fn serialize_example_data() {
        let example = GetEventInstancesCursorData::example_data();
        assert_eq!(json!(example), json!({"page": 4}));
    }

    #[test]
    fn deserialize_default() {
        let example = GetEventInstancesCursorData::default();
        assert_eq!(example, serde_json::from_value(json!({})).unwrap());
    }

    #[test]
    fn deserialize_example_data() {
        let example = GetEventInstancesCursorData::example_data();
        assert_eq!(example, serde_json::from_value(json!({"page": 4})).unwrap());
    }
}

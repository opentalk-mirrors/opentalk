// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{events::EventId, time::Timestamp, utils::ExampleData};

/// Data stored inside the `GET /events/instances` query cursor
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GetEventsAndInstancesCursorData {
    /// Last event in the list
    pub event_id: EventId,

    /// last event created at
    pub event_created_at: Timestamp,

    /// Last event starts_at
    pub event_starts_at: Option<Timestamp>,
}

impl ExampleData for GetEventsAndInstancesCursorData {
    fn example_data() -> Self {
        Self {
            event_id: EventId::example_data(),
            event_created_at: Timestamp::example_data(),
            event_starts_at: None,
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

    use super::GetEventsAndInstancesCursorData;
    use crate::pagination::Cursor;

    impl PartialSchema for GetEventsAndInstancesCursorData {
        fn schema() -> RefOr<Schema> {
            ObjectBuilder::new()
                .schema_type(Type::String)
                .description(Some("A cursor pointing to an event instance"))
                .examples([json!(Cursor(Self::example_data()))])
                .into()
        }
    }

    impl ToSchema for GetEventsAndInstancesCursorData {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

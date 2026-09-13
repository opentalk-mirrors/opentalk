// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{pagination::PageSize, time::Timestamp};

use super::GetEventInstancesCursorData;
use crate::pagination::Cursor;

/// Query parameters for the `GET /events/{event_id}/instances` endpoint
#[derive(Default, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams))]
pub struct GetEventInstancesQuery {
    /// Maximum number of invitees to include inside the event
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "super::serde_utils::invitees_max_or_zero"
        )
    )]
    pub invitees_max: Option<PageSize>,
    /// Minimum time of the event instances
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub time_min: Option<Timestamp>,
    /// Maximum time of the event instances
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub time_max: Option<Timestamp>,
    /// How many events to return per page
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub per_page: Option<PageSize>,
    /// Cursor token to get the next page of events
    #[cfg_attr(feature = "utoipa", param(schema_with = build_cursor_schema))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub after: Option<Cursor<GetEventInstancesCursorData>>,
}

#[cfg(feature = "utoipa")]
fn build_cursor_schema() -> impl Into<utoipa::openapi::RefOr<utoipa::openapi::Schema>> {
    use utoipa::PartialSchema as _;
    Cursor::<GetEventInstancesCursorData>::schema()
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::GetEventInstancesQuery;
    use crate::{events::GetEventInstancesCursorData, pagination::Cursor};

    #[test]
    fn serialize_default() {
        let example = GetEventInstancesQuery::default();
        assert_eq!(json!(example), json!({}));
    }

    #[test]
    fn serialize() {
        let cursor_data = GetEventInstancesCursorData {
            page: 26u64.try_into().unwrap(),
        };
        let example = GetEventInstancesQuery {
            invitees_max: Some(50.try_into().unwrap()),
            time_min: Some("2025-08-14T11:22:33Z".parse().unwrap()),
            time_max: Some("2025-09-14T12:23:34Z".parse().unwrap()),
            per_page: Some(30.try_into().unwrap()),
            after: Some(Cursor(cursor_data.clone())),
        };
        assert_eq!(
            json!(example),
            json!({
                "invitees_max": 50,
                "time_min": "2025-08-14T11:22:33Z",
                "time_max": "2025-09-14T12:23:34Z",
                "per_page": 30,
                "after": json!(Cursor(cursor_data)),
            })
        );
    }

    #[test]
    fn deserialize_default() {
        let example = GetEventInstancesQuery::default();
        assert_eq!(example, serde_json::from_value(json!({})).unwrap());
    }

    #[test]
    fn deserialize_invitees_max_zero() {
        let example = GetEventInstancesQuery {
            invitees_max: None,
            ..Default::default()
        };
        assert_eq!(
            example,
            serde_json::from_value(json!({"invitees_max": 0})).unwrap()
        );
    }

    #[test]
    fn deserialize() {
        let cursor_data = GetEventInstancesCursorData {
            page: 26u64.try_into().unwrap(),
        };
        let example = GetEventInstancesQuery {
            invitees_max: Some(65.try_into().unwrap()),
            time_min: Some("2025-08-14T11:22:33Z".parse().unwrap()),
            time_max: Some("2025-09-14T12:23:34Z".parse().unwrap()),
            per_page: Some(30.try_into().unwrap()),
            after: Some(Cursor(cursor_data.clone())),
        };
        assert_eq!(
            example,
            serde_json::from_value(json!({
                "invitees_max": 65,
                "time_min": "2025-08-14T11:22:33Z",
                "time_max": "2025-09-14T12:23:34Z",
                "per_page": 30,
                "after": json!(Cursor(cursor_data)),
            }))
            .unwrap()
        );
    }
}

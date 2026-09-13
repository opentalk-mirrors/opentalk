// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[cfg(feature = "serde")]
use opentalk_types_common::utils::comma_separated;
use opentalk_types_common::{
    events::invites::EventInviteStatus,
    pagination::{ItemCount, PageSize},
    time::Timestamp,
};

use crate::{events::GetEventsAndInstancesCursorData, pagination::Cursor};

/// Path query parameters of the `GET /events/instances` endpoint
///
/// Allows for customization in the search for events
#[derive(Default, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams))]
pub struct GetEventsAndInstancesQuery {
    /// Optional minimum time in which the event or instance happens
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub time_min: Option<Timestamp>,

    /// Optional maximum time in which the event or instance happens
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub time_max: Option<Timestamp>,

    /// Only query events created before this timestamp
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub created_before: Option<Timestamp>,

    /// Only query events created after this timestamp
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub created_after: Option<Timestamp>,

    /// Maximum number of invitees to return inside the event resource
    ///
    /// Default value is 0
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "super::serde_utils::invitees_max_or_zero"
        )
    )]
    pub invitees_max: Option<PageSize>,

    /// Maximum number of instances to return for each recurring event
    ///
    /// Default value is 0
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "ItemCount::is_default")
    )]
    pub instances_max: ItemCount,

    /// Return only favorite events
    #[cfg_attr(feature = "serde", serde(default))]
    pub favorites: bool,

    /// Filter the events by invite status
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            skip_serializing_if = "Vec::is_empty",
            with = "comma_separated",
        )
    )]
    pub invite_status: Vec<EventInviteStatus>,

    /// How many events and instances to return per page
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub per_page: Option<PageSize>,

    /// Cursor token to get the next page of events and instances
    ///
    /// Returned by the endpoint if the maximum number of events and instances per page has been hit
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub after: Option<Cursor<GetEventsAndInstancesCursorData>>,

    /// Only get events that are either marked as adhoc or non-adhoc
    ///
    /// If present, all adhoc events will be returned when `true`, all non-adhoc
    /// events will be returned when `false`. If not present, all events will
    /// be returned regardless of their `adhoc` flag value.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub adhoc: Option<bool>,

    /// Only get events that are either time-independent or time-dependent
    ///
    /// If present, all time-independent events will be returned when `true`,
    /// all time-dependent events will be returned when `false`. If absent,
    /// all events will be returned regardless of their time dependency.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub time_independent: Option<bool>,
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use opentalk_types_common::{events::invites::EventInviteStatus, utils::ExampleData};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::GetEventsAndInstancesQuery;
    use crate::{events::GetEventsAndInstancesCursorData, pagination::Cursor};

    fn example_cursor_data() -> GetEventsAndInstancesCursorData {
        GetEventsAndInstancesCursorData::example_data()
    }

    fn example() -> GetEventsAndInstancesQuery {
        GetEventsAndInstancesQuery {
            time_min: Some("2025-08-14T11:22:33Z".parse().unwrap()),
            time_max: Some("2025-09-14T12:23:34Z".parse().unwrap()),
            created_after: Some("2024-08-14T11:22:33Z".parse().unwrap()),
            created_before: Some("2025-06-14T12:23:34Z".parse().unwrap()),
            invitees_max: Some(50.try_into().unwrap()),
            instances_max: 10.into(),
            favorites: true,
            invite_status: vec![EventInviteStatus::Accepted, EventInviteStatus::Pending],
            per_page: Some(100u64.try_into().unwrap()),
            after: Some(Cursor(example_cursor_data())),
            adhoc: Some(false),
            time_independent: Some(true),
        }
    }

    fn example_json() -> serde_json::Value {
        json!({
            "time_min": "2025-08-14T11:22:33Z",
            "time_max": "2025-09-14T12:23:34Z",
            "created_after": "2024-08-14T11:22:33Z",
            "created_before": "2025-06-14T12:23:34Z",
            "invitees_max": 50,
            "instances_max": 10,
            "favorites": true,
            "invite_status": "accepted,pending",
            "per_page": 100,
            "after": json!(Cursor(example_cursor_data())),
            "adhoc": false,
            "time_independent": true,

        })
    }

    #[test]
    fn serialize_default() {
        let example = GetEventsAndInstancesQuery::default();
        assert_eq!(json!(example), json!({"favorites": false}));
    }

    #[test]
    fn serialize() {
        assert_eq!(json!(example()), example_json());
    }

    #[test]
    fn deserialize_default() {
        let example = GetEventsAndInstancesQuery::default();
        assert_eq!(example, serde_json::from_value(json!({})).unwrap());
    }

    #[test]
    fn deserialize_invitees_max_zero() {
        let example = GetEventsAndInstancesQuery {
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
        assert_eq!(example(), serde_json::from_value(example_json()).unwrap());
    }
}

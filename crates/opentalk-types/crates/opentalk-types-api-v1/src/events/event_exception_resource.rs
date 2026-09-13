// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{TimeZone as _, Utc};
use opentalk_types_common::{
    events::{EventDescription, EventId, EventTitle},
    time::{DateTimeTz, Timestamp},
    utils::ExampleData,
};

use crate::{
    events::{EventAndInstanceId, EventStatus, ExceptionMarker, InstanceId},
    users::PublicUserProfile,
};

/// Event exception resource
///
/// Overrides event properties for a event recurrence. May only exist for events of type `recurring`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(EventExceptionResource::example_data()))
)]
pub struct EventExceptionResource {
    /// Opaque ID of the exception
    pub id: EventAndInstanceId,

    /// ID of the event  the exception belongs to
    pub recurring_event_id: EventId,

    /// ID of the instance the exception overrides
    pub instance_id: InstanceId,

    /// Public user profile of the user which created the exception
    pub created_by: PublicUserProfile,

    /// Timestamp of the exceptions creation
    pub created_at: Timestamp,

    /// Public user profile of the user which last updated the exception
    pub updated_by: PublicUserProfile,

    /// Timestamp of the exceptions last update
    pub updated_at: Timestamp,

    /// Override the title of the instance
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub title: Option<EventTitle>,

    /// Override the description of the instance
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub description: Option<EventDescription>,

    /// Override the `is_all_day` property of the instance
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub is_all_day: Option<bool>,

    /// Override the `starts_at` time of the instance
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub starts_at: Option<DateTimeTz>,

    /// Override the `ends_at` time of the instance
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub ends_at: Option<DateTimeTz>,

    /// The `starts_at` of the instance this exception modifies. Used to match the exception the instance
    pub original_starts_at: DateTimeTz,

    /// Must always be `exception`
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub type_: ExceptionMarker,

    /// Override the status of the event instance
    ///
    /// This can be used to cancel a occurrence of an event
    pub status: EventStatus,

    /// Can the current user edit this resource
    pub can_edit: bool,
}

impl ExampleData for EventExceptionResource {
    fn example_data() -> Self {
        Self {
            id: EventAndInstanceId::example_data(),
            recurring_event_id: EventId::example_data(),
            instance_id: InstanceId::example_data(),
            created_by: PublicUserProfile::example_data(),
            created_at: Timestamp::example_data(),
            updated_by: PublicUserProfile::example_data(),
            updated_at: Timestamp::example_data(),
            title: Some(EventTitle::example_data()),
            description: Some(EventDescription::example_data()),
            is_all_day: Some(false),
            starts_at: Some(DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2024, 7, 5, 15, 0, 0).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            }),
            ends_at: Some(DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2024, 7, 5, 17, 0, 0).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            }),
            original_starts_at: DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2024, 7, 5, 16, 0, 0).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            },
            type_: ExceptionMarker::Exception,
            status: EventStatus::Ok,
            can_edit: false,
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn roundtrip() {
        let deserialized = EventExceptionResource {
            id: EventAndInstanceId::example_data(),
            recurring_event_id: EventId::example_data(),
            instance_id: InstanceId::example_data(),
            created_by: PublicUserProfile::example_data(),
            created_at: Timestamp::example_data(),
            updated_by: PublicUserProfile::example_data(),
            updated_at: Timestamp::example_data(),
            title: Some(EventTitle::example_data()),
            description: Some(EventDescription::example_data()),
            is_all_day: Some(false),
            starts_at: Some(DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2024, 7, 5, 15, 0, 0).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            }),
            ends_at: Some(DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2024, 7, 5, 17, 0, 0).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            }),
            original_starts_at: DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2024, 7, 5, 16, 0, 0).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            },
            type_: ExceptionMarker::Exception,
            status: EventStatus::Ok,
            can_edit: false,
        };

        let serialized = json!({
            "can_edit": false,
            "created_at": "2024-07-20T14:16:19Z",
            "created_by":  {
                "avatar_url": "https://gravatar.com/avatar/c160f8cc69a4f0bf2b0362752353d060",
                "display_name": "Alice Adams",
                "email": "alice@example.com",
                "firstname": "Alice",
                "id": "00000000-0000-0000-0000-0000000a11c3",
                "lastname": "Adams",
                "title": "",
            },
            "description": "The Weekly Team Event",
            "ends_at":  {
                "datetime": "2024-07-05T17:00:00Z",
                "timezone": "Europe/Berlin",
            },
            "id": "00000000-0000-0000-0000-004433221100_20240705T170242Z",
            "instance_id": "20240705T170242Z",
            "is_all_day": false,
            "original_starts_at":  {
                "datetime": "2024-07-05T16:00:00Z",
                "timezone": "Europe/Berlin",
            },
            "recurring_event_id": "00000000-0000-0000-0000-004433221100",
            "starts_at":  {
                "datetime": "2024-07-05T15:00:00Z",
                "timezone": "Europe/Berlin",
            },
            "status": "ok",
            "title": "Team Event",
            "type": "exception",
            "updated_at": "2024-07-20T14:16:19Z",
            "updated_by":  {
                "avatar_url": "https://gravatar.com/avatar/c160f8cc69a4f0bf2b0362752353d060",
                "display_name": "Alice Adams",
                "email": "alice@example.com",
                "firstname": "Alice",
                "id": "00000000-0000-0000-0000-0000000a11c3",
                "lastname": "Adams",
                "title": "",
            },
        });

        assert_eq!(serde_json::to_value(&deserialized).unwrap(), serialized);
        assert_eq!(
            serde_json::from_value::<EventExceptionResource>(serialized).unwrap(),
            deserialized
        );
    }
}

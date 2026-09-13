// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    events::{EventDescription, EventId, EventTitle, invites::EventInviteStatus},
    shared_folders::SharedFolder,
    time::{DateTimeTz, Timestamp},
    training_participation_report::TrainingParticipationReportParameterSet,
    utils::ExampleData,
};

use crate::{
    events::{
        EventAndInstanceId, EventInvitee, EventRoomInfo, EventStatus, InstanceId, InstanceMarker,
    },
    users::PublicUserProfile,
};

/// Event instance resource
///
/// An event instance is an occurrence of an recurring event
///
/// Exceptions for the instance are always already applied
///
/// For infos on undocumented fields see [`EventResource`]
///
/// [`EventResource`]: ../event_ressource/struct.EventResource.html
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(EventInstance::example_data())),
)]
pub struct EventInstance {
    /// Opaque id of the event instance resource
    pub id: EventAndInstanceId,

    /// ID of the recurring event this instance belongs to
    pub recurring_event_id: EventId,

    /// Opaque id of the instance
    pub instance_id: InstanceId,

    /// Public user profile of the user which created the event
    pub created_by: PublicUserProfile,

    /// Timestamp of the event creation
    pub created_at: Timestamp,

    /// Public user profile of the user which last updated the event
    /// or created the exception which modified the instance
    pub updated_by: PublicUserProfile,

    /// Timestamp of the last update
    pub updated_at: Timestamp,

    /// Title of the event
    pub title: EventTitle,
    /// Description of the event
    pub description: EventDescription,
    /// All information about the room the event takes place in
    pub room: EventRoomInfo,
    /// Flag which indicates if `invitees` contains all invites as far as known to the application
    pub invitees_truncated: bool,
    /// List of event invitees and their invite status. Might not be complete, see `invite_truncated`
    pub invitees: Vec<EventInvitee>,
    /// Flag indicating whether the event is all-day
    pub is_all_day: bool,
    /// Start time of the event.
    pub starts_at: DateTimeTz,
    /// End time of the event.
    pub ends_at: DateTimeTz,

    /// Must always be `instance`
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub type_: InstanceMarker,
    /// The invite status of the current user for this event
    pub status: EventStatus,
    /// Is this event in the current user's favorite list?
    pub invite_status: EventInviteStatus,
    /// Flag to indicate if the event is a favorite of the current user
    pub is_favorite: bool,
    /// Fkag to indicate if the current user can edit the event
    pub can_edit: bool,

    /// Information about the shared folder for the event
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub shared_folder: Option<SharedFolder>,

    /// The training participation report parameter set for the event.
    ///
    /// When present, the training participation report will be started
    /// automatically in the meeting.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub training_participation_report: Option<TrainingParticipationReportParameterSet>,
}

impl ExampleData for EventInstance {
    fn example_data() -> Self {
        use chrono::{TimeZone as _, Utc};

        Self {
            id: EventAndInstanceId::example_data(),
            recurring_event_id: EventId::example_data(),
            instance_id: InstanceId::example_data(),
            created_by: PublicUserProfile::example_data(),
            created_at: Timestamp::example_data(),
            updated_by: PublicUserProfile::example_data(),
            updated_at: Timestamp::example_data(),
            title: EventTitle::example_data(),
            description: EventDescription::example_data(),
            room: EventRoomInfo::example_data(),
            invitees_truncated: true,
            invitees: vec![EventInvitee::example_data()],
            is_all_day: false,
            starts_at: DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2024, 7, 22, 10, 0, 0).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            },
            ends_at: DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2024, 7, 22, 11, 0, 0).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            },
            type_: InstanceMarker::Instance,
            status: EventStatus::Ok,
            invite_status: EventInviteStatus::Pending,
            is_favorite: false,
            can_edit: false,
            shared_folder: Some(SharedFolder::example_data()),
            training_participation_report: Some(
                TrainingParticipationReportParameterSet::example_data(),
            ),
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use chrono::{TimeZone, Utc};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn roundtrip() {
        let deserialized = EventInstance {
            id: EventAndInstanceId::example_data(),
            recurring_event_id: EventId::example_data(),
            instance_id: InstanceId::example_data(),
            created_by: PublicUserProfile::example_data(),
            created_at: Timestamp::example_data(),
            updated_by: PublicUserProfile::example_data(),
            updated_at: Timestamp::example_data(),
            title: EventTitle::example_data(),
            description: EventDescription::example_data(),
            room: EventRoomInfo::example_data(),
            invitees_truncated: true,
            invitees: vec![EventInvitee::example_data()],
            is_all_day: false,
            starts_at: DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2024, 7, 22, 10, 0, 0).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            },
            ends_at: DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2024, 7, 22, 11, 0, 0).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            },
            type_: InstanceMarker::Instance,
            status: EventStatus::Ok,
            invite_status: EventInviteStatus::Pending,
            is_favorite: false,
            can_edit: false,
            shared_folder: Some(SharedFolder::example_data()),
            training_participation_report: Some(
                TrainingParticipationReportParameterSet::example_data(),
            ),
        };

        let serialized = json!({
            "can_edit": false,
            "created_at": "2024-07-20T14:16:19Z",
            "created_by": {
                "avatar_url": "https://gravatar.com/avatar/c160f8cc69a4f0bf2b0362752353d060",
                "display_name": "Alice Adams",
                "email": "alice@example.com",
                "firstname": "Alice",
                "id": "00000000-0000-0000-0000-0000000a11c3",
                "lastname": "Adams",
                "title": "",
            },
            "description": "The Weekly Team Event",
            "ends_at": {
                "datetime": "2024-07-22T11:00:00Z",
                "timezone": "Europe/Berlin",
            },
            "id": "00000000-0000-0000-0000-004433221100_20240705T170242Z",
            "instance_id": "20240705T170242Z",
            "invite_status": "pending",
            "invitees": [
                {
                    "profile": {
                        "avatar_url": "https://gravatar.com/avatar/c160f8cc69a4f0bf2b0362752353d060",
                        "display_name": "Alice Adams",
                        "email": "alice@example.com",
                        "firstname": "Alice",
                        "id": "00000000-0000-0000-0000-0000000a11c3",
                        "kind": "registered",
                        "lastname": "Adams",
                        "role": "user",
                        "title": "",
                    },
                    "status": "accepted",
                },
            ],
            "invitees_truncated": true,
            "is_all_day": false,
            "is_favorite": false,
            "recurring_event_id": "00000000-0000-0000-0000-004433221100",
            "room": {
                "call_in": {
                    "id": "1234567890",
                    "password": "9876543210",
                    "tel": "+555-12345-67890",
                },
                "e2e_encryption": false,
                "id": "00000000-0000-0000-0000-0000abadcafe",
                "alias": "personal-room-name_0000000000000000",
                "password": "v3rys3cr3t",
                "waiting_room": false,
                "guest_access": "waiting_room",
            },
            "shared_folder":  {
                "read":  {
                    "password": "v3rys3cr3t",
                    "url": "https://cloud.example.com/shares/abc123",
                },
            },
            "starts_at":  {
                "datetime": "2024-07-22T10:00:00Z",
                "timezone": "Europe/Berlin",
            },
            "status": "ok",
            "title": "Team Event",
            "training_participation_report":  {
                "checkpoint_interval":  {
                    "after": 300,
                    "within": 400,
                },
                "initial_checkpoint_delay":  {
                    "after": 100,
                    "within": 200,
                },
            },
            "type": "instance",
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
            serde_json::from_value::<EventInstance>(serialized).unwrap(),
            deserialized
        );
    }
}

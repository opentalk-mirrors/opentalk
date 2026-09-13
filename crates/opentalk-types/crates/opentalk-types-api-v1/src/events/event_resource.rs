// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    events::{EventDescription, EventId, EventTitle, invites::EventInviteStatus},
    shared_folders::SharedFolder,
    streaming::RoomStreamingTarget,
    time::Timestamp,
    training_participation_report::TrainingParticipationReportParameterSet,
    utils::ExampleData,
};

use super::{EventInvitee, EventRoomInfo};
use crate::{events::EventResourceDateKind, users::PublicUserProfile};

/// Event Resource representation
///
/// Returned from `GET /events/` and `GET /events/{event_id}`
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(EventResource::example_data()))
)]
pub struct EventResource {
    /// ID of the event
    pub id: EventId,

    /// Public user profile of the user which created the event
    pub created_by: PublicUserProfile,

    /// Timestamp of the event creation
    pub created_at: Timestamp,

    /// Public user profile of the user which last updated the event
    pub updated_by: PublicUserProfile,

    /// Timestamp of the last update
    pub updated_at: Timestamp,

    /// Title of the event
    ///
    /// For display purposes
    pub title: EventTitle,

    /// Description of the event
    ///
    /// For display purposes
    pub description: EventDescription,

    /// All information about the room the event takes place in
    pub room: EventRoomInfo,

    /// Flag which indicates if `invitees` contains all invites as far as known to the application
    /// May also be true if there are no invitees but no invitees were requested
    pub invitees_truncated: bool,

    /// List of event invitees and their invite status. Might not be complete, see `invite_truncated`
    pub invitees: Vec<EventInvitee>,

    /// Flag indicating whether the event is ad-hoc created.
    pub is_adhoc: bool,

    /// The invite status of the current user for this event
    pub invite_status: EventInviteStatus,

    /// Is this event in the current user's favorite list?
    pub is_favorite: bool,

    /// Can the current user edit this resource
    pub can_edit: bool,

    /// Information about the shared folder for the event
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub shared_folder: Option<SharedFolder>,

    /// The streaming targets of the room associated with the event
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub streaming_targets: Vec<RoomStreamingTarget>,

    /// Indicates whether meeting details should be provided. If absent, no meeting details are made available.
    pub show_meeting_details: bool,

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

    /// The field containing optional parameter related to the date of the event.
    #[cfg_attr(feature = "serde", serde(flatten,))]
    pub date: EventResourceDateKind,
}

impl ExampleData for EventResource {
    fn example_data() -> Self {
        Self {
            id: EventId::example_data(),
            created_by: PublicUserProfile::example_data(),
            created_at: Timestamp::example_data(),
            updated_by: PublicUserProfile::example_data(),
            updated_at: Timestamp::example_data(),
            title: EventTitle::example_data(),
            description: EventDescription::example_data(),
            room: EventRoomInfo::example_data(),
            invitees_truncated: false,
            invitees: vec![EventInvitee::example_data()],
            date: EventResourceDateKind::example_data(),
            is_adhoc: false,
            invite_status: EventInviteStatus::Accepted,
            is_favorite: false,
            can_edit: false,
            shared_folder: Some(SharedFolder::example_data()),
            streaming_targets: vec![RoomStreamingTarget::example_data()],
            show_meeting_details: true,
            training_participation_report: Some(
                TrainingParticipationReportParameterSet::example_data(),
            ),
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use chrono::{TimeZone, Utc};
    use opentalk_types_common::time::{DateTimeTz, RecurrencePattern};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::events::EventResourceDate;

    #[test]
    fn serialize_without_options() {
        let expected = json!({
            "id": EventId::example_data(),
            "created_by": PublicUserProfile::example_data(),
            "created_at": Timestamp::example_data(),
            "updated_by": PublicUserProfile::example_data(),
            "updated_at": Timestamp::example_data(),
            "title": EventTitle::example_data(),
            "description": EventDescription::example_data(),
            "room": EventRoomInfo::example_data(),
            "invitees_truncated": false,
            "invitees": [],
            "is_time_independent": true,
            "is_adhoc": false,
            "type": "single",
            "invite_status": "accepted",
            "is_favorite": false,
            "can_edit": false,
            "show_meeting_details": true,
        });

        let produced = json!(EventResource {
            id: EventId::example_data(),
            created_by: PublicUserProfile::example_data(),
            created_at: Timestamp::example_data(),
            updated_by: PublicUserProfile::example_data(),
            updated_at: Timestamp::example_data(),
            title: EventTitle::example_data(),
            description: EventDescription::example_data(),
            room: EventRoomInfo::example_data(),
            invitees_truncated: false,
            invitees: vec![],
            date: EventResourceDateKind::TIME_INDEPENDENT,
            is_adhoc: false,
            invite_status: EventInviteStatus::Accepted,
            is_favorite: false,
            can_edit: false,
            shared_folder: None,
            streaming_targets: vec![],
            show_meeting_details: true,
            training_participation_report: None,
        });

        assert_eq!(expected, produced);
    }

    #[test]
    fn deserialize_without_options() {
        let expected = EventResource {
            id: EventId::example_data(),
            created_by: PublicUserProfile::example_data(),
            created_at: Timestamp::example_data(),
            updated_by: PublicUserProfile::example_data(),
            updated_at: Timestamp::example_data(),
            title: EventTitle::example_data(),
            description: EventDescription::example_data(),
            room: EventRoomInfo::example_data(),
            invitees_truncated: false,
            invitees: vec![],
            date: EventResourceDateKind::TIME_INDEPENDENT,
            is_adhoc: false,
            invite_status: EventInviteStatus::Accepted,
            is_favorite: false,
            can_edit: false,
            shared_folder: None,
            streaming_targets: vec![],
            show_meeting_details: true,
            training_participation_report: None,
        };

        let produced = serde_json::from_value(json!({
            "id": EventId::example_data(),
            "created_by": PublicUserProfile::example_data(),
            "created_at": Timestamp::example_data(),
            "updated_by": PublicUserProfile::example_data(),
            "updated_at": Timestamp::example_data(),
            "title": EventTitle::example_data(),
            "description": EventDescription::example_data(),
            "room": EventRoomInfo::example_data(),
            "invitees_truncated": false,
            "invitees": [],
            "is_time_independent": true,
            "is_adhoc": false,
            "type": "single",
            "invite_status": "accepted",
            "is_favorite": false,
            "can_edit": false,
            "show_meeting_details": true,
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_with_options() {
        let expected = json!({
            "id": EventId::example_data(),
            "created_by": PublicUserProfile::example_data(),
            "created_at": Timestamp::example_data(),
            "updated_by": PublicUserProfile::example_data(),
            "updated_at": Timestamp::example_data(),
            "title": EventTitle::example_data(),
            "description": EventDescription::example_data(),
            "room": EventRoomInfo::example_data(),
            "invitees_truncated": false,
            "invitees": [EventInvitee::example_data()],
            "is_time_independent": false,
            "is_all_day": false,
            "starts_at": {
                "datetime": "2002-04-01T10:41:35Z",
                "timezone": "Europe/Berlin",
            },
            "ends_at": {
                "datetime": "2002-04-01T11:41:35Z",
                "timezone": "Europe/Berlin",
            },
            "recurrence_pattern": RecurrencePattern::example_data(),
            "is_adhoc": false,
            "type": "recurring",
            "invite_status": "accepted",
            "is_favorite": false,
            "can_edit": false,
            "shared_folder": SharedFolder::example_data(),
            "streaming_targets": [RoomStreamingTarget::example_data()],
            "show_meeting_details": true,
            "training_participation_report": TrainingParticipationReportParameterSet::example_data(),
        });

        let produced = json!(EventResource {
            id: EventId::example_data(),
            created_by: PublicUserProfile::example_data(),
            created_at: Timestamp::example_data(),
            updated_by: PublicUserProfile::example_data(),
            updated_at: Timestamp::example_data(),
            title: EventTitle::example_data(),
            description: EventDescription::example_data(),
            room: EventRoomInfo::example_data(),
            invitees_truncated: false,
            invitees: vec![EventInvitee::example_data()],
            date: EventResourceDate::Recurring {
                is_all_day: false,
                starts_at: DateTimeTz {
                    datetime: Utc.with_ymd_and_hms(2002, 4, 1, 10, 41, 35).unwrap(),
                    timezone: chrono_tz::Europe::Berlin.into(),
                },
                ends_at: DateTimeTz {
                    datetime: Utc.with_ymd_and_hms(2002, 4, 1, 11, 41, 35).unwrap(),
                    timezone: chrono_tz::Europe::Berlin.into(),
                },
                recurrence_pattern: RecurrencePattern::example_data(),
            }
            .into(),
            is_adhoc: false,
            invite_status: EventInviteStatus::Accepted,
            is_favorite: false,
            can_edit: false,
            shared_folder: Some(SharedFolder::example_data()),
            streaming_targets: vec![RoomStreamingTarget::example_data()],
            show_meeting_details: true,
            training_participation_report: Some(
                TrainingParticipationReportParameterSet::example_data()
            ),
        });

        assert_eq!(expected, produced);
    }

    #[test]
    fn deserialize_with_options() {
        let expected = EventResource {
            id: EventId::example_data(),
            created_by: PublicUserProfile::example_data(),
            created_at: Timestamp::example_data(),
            updated_by: PublicUserProfile::example_data(),
            updated_at: Timestamp::example_data(),
            title: EventTitle::example_data(),
            description: EventDescription::example_data(),
            room: EventRoomInfo::example_data(),
            invitees_truncated: false,
            invitees: vec![EventInvitee::example_data()],
            date: EventResourceDate::Recurring {
                is_all_day: false,
                starts_at: DateTimeTz {
                    datetime: Utc.with_ymd_and_hms(2002, 4, 1, 10, 41, 35).unwrap(),
                    timezone: chrono_tz::Europe::Berlin.into(),
                },
                ends_at: DateTimeTz {
                    datetime: Utc.with_ymd_and_hms(2002, 4, 1, 11, 41, 35).unwrap(),
                    timezone: chrono_tz::Europe::Berlin.into(),
                },
                recurrence_pattern: RecurrencePattern::example_data(),
            }
            .into(),
            is_adhoc: false,
            invite_status: EventInviteStatus::Accepted,
            is_favorite: false,
            can_edit: false,
            shared_folder: Some(SharedFolder::example_data()),
            streaming_targets: vec![RoomStreamingTarget::example_data()],
            show_meeting_details: true,
            training_participation_report: Some(
                TrainingParticipationReportParameterSet::example_data(),
            ),
        };

        let produced = serde_json::from_value(json!({
            "id": EventId::example_data(),
            "created_by": PublicUserProfile::example_data(),
            "created_at": Timestamp::example_data(),
            "updated_by": PublicUserProfile::example_data(),
            "updated_at": Timestamp::example_data(),
            "title": EventTitle::example_data(),
            "description": EventDescription::example_data(),
            "room": EventRoomInfo::example_data(),
            "invitees_truncated": false,
            "invitees": [EventInvitee::example_data()],
            "is_time_independent": false,
            "is_all_day": false,
            "starts_at": {
                "datetime": "2002-04-01T10:41:35Z",
                "timezone": "Europe/Berlin",
            },
            "ends_at": {
                "datetime": "2002-04-01T11:41:35Z",
                "timezone": "Europe/Berlin",
            },
            "recurrence_pattern": RecurrencePattern::example_data(),
            "is_adhoc": false,
            "type": "recurring",
            "invite_status": "accepted",
            "is_favorite": false,
            "can_edit": false,
            "shared_folder": SharedFolder::example_data(),
            "streaming_targets": [RoomStreamingTarget::example_data()],
            "show_meeting_details": true,
            "training_participation_report": TrainingParticipationReportParameterSet::example_data(),
        })).unwrap();

        assert_eq!(expected, produced);
    }
}

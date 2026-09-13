// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{TimeZone, Utc};
use opentalk_types_common::{
    events::{EventDescription, EventTitle},
    rooms::{RoomAlias, RoomPassword},
    shared_folders::SharedFolder,
    streaming::RoomStreamingTarget,
    users::{Language, UserTitle},
    utils::ExampleData,
};
use uuid::Uuid;

mod invites;

pub use invites::{
    ExternalEventCancellation, ExternalEventInvite, ExternalEventUninvite, ExternalEventUpdate,
    RegisteredEventCancellation, RegisteredEventInvite, RegisteredEventUninvite,
    RegisteredEventUpdate, UnregisteredEventCancellation, UnregisteredEventInvite,
    UnregisteredEventUninvite, UnregisteredEventUpdate,
};

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(Email::example_data()))
)]
pub struct Email(String);

impl ExampleData for Email {
    fn example_data() -> Self {
        Email::from("alice.adams@example.com")
    }
}

impl Email {
    pub fn new(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Email {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl From<String> for Email {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(RegisteredUser::example_data()))
)]
pub struct RegisteredUser {
    pub email: Email,
    pub title: UserTitle,
    pub first_name: String,
    pub last_name: String,
    pub language: Language,
}

impl ExampleData for RegisteredUser {
    fn example_data() -> Self {
        Self {
            email: Email::from("alice.adams@example.com"),
            title: "Dr.".parse().expect("valid user title"),
            first_name: "Alice".to_string(),
            last_name: "Adams".to_string(),
            language: "en".parse().expect("valid language"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(UnregisteredUser::example_data()))
)]
pub struct UnregisteredUser {
    pub email: Email,
    pub first_name: String,
    pub last_name: String,
}

impl ExampleData for UnregisteredUser {
    fn example_data() -> Self {
        Self {
            email: Email::from("bob.burton@example.com"),
            first_name: "Bob".to_string(),
            last_name: "Burton".to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(ExternalUser::example_data()))
)]
pub struct ExternalUser {
    pub email: Email,
}

impl ExampleData for ExternalUser {
    fn example_data() -> Self {
        Self {
            email: Email::from("charlie.cooper@example.com"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum User {
    Registered(RegisteredUser),
    Unregistered(UnregisteredUser),
    External(ExternalUser),
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Time {
    pub time: chrono::DateTime<Utc>,
    pub timezone: String,
}

impl ExampleData for Time {
    fn example_data() -> Self {
        Self {
            time: Utc.with_ymd_and_hms(2024, 7, 5, 17, 2, 42).unwrap(),
            timezone: "Europe/Berlin".to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(Event::example_data()))
)]
pub struct Event {
    pub id: Uuid,
    pub name: EventTitle,
    pub created_at: Time,
    pub start_time: Option<Time>,
    pub end_time: Option<Time>,
    pub rrule: Option<String>,
    pub description: EventDescription,
    pub room: Room,
    pub call_in: Option<CallIn>,
    pub revision: i32,
    pub shared_folder: Option<SharedFolder>,
    pub adhoc_retention_seconds: Option<u64>,
    pub streaming_targets: Vec<RoomStreamingTarget>,
}

impl ExampleData for Event {
    fn example_data() -> Self {
        Self {
            id: Uuid::from_u128(0xabadcafe),
            name: "Weekly teammeeting".parse().expect("valid event title"),
            created_at: Time::example_data(),
            start_time: None,
            end_time: None,
            rrule: None,
            description: "The team's regular weekly meeting"
                .parse()
                .expect("valid event description"),
            room: Room::example_data(),
            call_in: Some(CallIn::example_data()),
            revision: 3,
            shared_folder: Some(SharedFolder::example_data()),
            adhoc_retention_seconds: None,
            streaming_targets: vec![],
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(EventException::example_data()))
)]
pub struct EventException {
    pub exception_date: Time,
    pub kind: EventExceptionKind,
    pub title: Option<EventTitle>,
    pub description: Option<EventDescription>,
    pub is_all_day: Option<bool>,
    pub starts_at: Option<Time>,
    pub ends_at: Option<Time>,
}

impl ExampleData for EventException {
    fn example_data() -> Self {
        Self {
            exception_date: Time::example_data(),
            kind: EventExceptionKind::Modified,
            title: Some("Another weekly meeting".parse().expect("valid event title")),
            description: None,
            is_all_day: None,
            starts_at: None,
            ends_at: None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(EventExceptionKind::Modified))
)]
pub enum EventExceptionKind {
    Modified,
    Canceled,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(Room::example_data()))
)]
pub struct Room {
    pub id: Uuid,
    pub alias: Option<RoomAlias>,
    pub password: Option<RoomPassword>,
}

impl ExampleData for Room {
    fn example_data() -> Self {
        Self {
            id: Uuid::from_u128(0xabcdef99),
            alias: Some(RoomAlias::example_data()),
            password: Some("v3rys3cr3t".parse().unwrap()),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(CallIn::example_data())),
)]
pub struct CallIn {
    pub sip_tel: String,
    pub sip_id: String,
    pub sip_password: String,
}

impl ExampleData for CallIn {
    fn example_data() -> Self {
        Self {
            sip_tel: "+99-1234567890".to_string(),
            sip_id: "1234567890".to_string(),
            sip_password: "9876543210".to_string(),
        }
    }
}

/// The different kinds of MailTasks that are currently supported
#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(tag = "message", rename_all = "snake_case")
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(as = v1::Message))]
pub enum Message {
    // Invites
    RegisteredEventInvite(RegisteredEventInvite),
    UnregisteredEventInvite(UnregisteredEventInvite),
    ExternalEventInvite(ExternalEventInvite),
    // Updates
    RegisteredEventUpdate(RegisteredEventUpdate),
    UnregisteredEventUpdate(UnregisteredEventUpdate),
    ExternalEventUpdate(ExternalEventUpdate),
    // Cancellations
    RegisteredEventCancellation(RegisteredEventCancellation),
    UnregisteredEventCancellation(UnregisteredEventCancellation),
    ExternalEventCancellation(ExternalEventCancellation),
    // Uninvites
    RegisteredEventUninvite(RegisteredEventUninvite),
    UnregisteredEventUninvite(UnregisteredEventUninvite),
    ExternalEventUninvite(ExternalEventUninvite),
}

impl From<RegisteredEventInvite> for Message {
    fn from(value: RegisteredEventInvite) -> Self {
        Message::RegisteredEventInvite(value)
    }
}

impl From<UnregisteredEventInvite> for Message {
    fn from(value: UnregisteredEventInvite) -> Self {
        Message::UnregisteredEventInvite(value)
    }
}

impl From<ExternalEventInvite> for Message {
    fn from(value: ExternalEventInvite) -> Self {
        Message::ExternalEventInvite(value)
    }
}

impl From<RegisteredEventUpdate> for Message {
    fn from(value: RegisteredEventUpdate) -> Self {
        Message::RegisteredEventUpdate(value)
    }
}

impl From<UnregisteredEventUpdate> for Message {
    fn from(value: UnregisteredEventUpdate) -> Self {
        Message::UnregisteredEventUpdate(value)
    }
}

impl From<ExternalEventUpdate> for Message {
    fn from(value: ExternalEventUpdate) -> Self {
        Message::ExternalEventUpdate(value)
    }
}

impl From<RegisteredEventCancellation> for Message {
    fn from(value: RegisteredEventCancellation) -> Self {
        Message::RegisteredEventCancellation(value)
    }
}

impl From<UnregisteredEventCancellation> for Message {
    fn from(value: UnregisteredEventCancellation) -> Self {
        Message::UnregisteredEventCancellation(value)
    }
}

impl From<ExternalEventCancellation> for Message {
    fn from(value: ExternalEventCancellation) -> Self {
        Message::ExternalEventCancellation(value)
    }
}

impl From<RegisteredEventUninvite> for Message {
    fn from(value: RegisteredEventUninvite) -> Self {
        Message::RegisteredEventUninvite(value)
    }
}

impl From<UnregisteredEventUninvite> for Message {
    fn from(value: UnregisteredEventUninvite) -> Self {
        Message::UnregisteredEventUninvite(value)
    }
}

impl From<ExternalEventUninvite> for Message {
    fn from(value: ExternalEventUninvite) -> Self {
        Message::ExternalEventUninvite(value)
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use chrono::FixedOffset;
    use opentalk_types_common::shared_folders::SharedFolderAccess;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::*;

    #[test]
    fn test_basic_format() {
        let basic_invite = MailTask::V1(Message::RegisteredEventInvite(RegisteredEventInvite {
            inviter: RegisteredUser {
                email: "bob@example.org".into(),
                title: "Prof. Dr.".parse().expect("valid user title"),
                first_name: "Bob".into(),
                last_name: "Inviter".into(),
                language: "de".parse().expect("valid language"),
            },
            event: Event {
                id: Uuid::from_u128(1),
                name: "Guten Morgen Meeting".parse().expect("valid event title"),
                description: "".parse().expect("valid event description"),
                created_at: Time {
                    time: chrono::DateTime::<FixedOffset>::parse_from_rfc3339(
                        "2021-12-29T15:00:00+00:00",
                    )
                    .unwrap()
                    .into(),
                    timezone: "UTC".into(),
                },
                start_time: Some(Time {
                    time: chrono::DateTime::<FixedOffset>::parse_from_rfc3339(
                        "2021-12-29T15:00:00+02:00",
                    )
                    .unwrap()
                    .into(),
                    timezone: "Europe/Berlin".into(),
                }),
                end_time: Some(Time {
                    time: chrono::DateTime::<FixedOffset>::parse_from_rfc3339(
                        "2021-12-29T15:30:00+02:00",
                    )
                    .unwrap()
                    .into(),
                    timezone: "Europe/Berlin".into(),
                }),
                rrule: None,
                room: Room {
                    id: Uuid::from_u128(0),
                    alias: Some(RoomAlias::example_data()),
                    password: Some("password123".parse().unwrap()),
                },
                call_in: Some(CallIn {
                    sip_tel: "+497652917".into(),
                    sip_id: "2".into(),
                    sip_password: "987".into(),
                }),
                revision: 0,
                shared_folder: Some(SharedFolder {
                    read: SharedFolderAccess {
                        url: "https://nextcloud.example.com/s/TArrLyC3K7c5Jbg".to_string(),
                        password: "DLgoYrFEoy".to_string(),
                    },
                    read_write: None,
                }),
                adhoc_retention_seconds: Some(86400),
                streaming_targets: Vec::new(),
            },
            invitee: RegisteredUser {
                email: "lastname@example.org".into(),
                title: "Prof. Dr.".parse().expect("valid user title"),
                first_name: "FirstName".into(),
                last_name: "LastName".into(),
                language: "de".parse().expect("valid language"),
            },
        }));

        assert_eq!(
            basic_invite,
            serde_json::from_value(serde_json::json!({
                "version": "1",
                "message": "registered_event_invite",
                "event": {
                    "id": Uuid::from_u128(1),
                    "name": "Guten Morgen Meeting",
                    "description": "",
                    "created_at": {"time":"2021-12-29T15:00:00+00:00", "timezone": "UTC"},
                    "start_time": {"time":"2021-12-29T15:00:00+02:00", "timezone": "Europe/Berlin"},
                    "end_time": {"time": "2021-12-29T15:30:00+02:00", "timezone": "Europe/Berlin"},
                    "room": {
                        "id": Uuid::from_u128(0),
                        "alias": RoomAlias::example_data(),
                        "password": "password123"
                    },
                    "call_in": {
                        "sip_tel": "+497652917",
                        "sip_id": "2",
                        "sip_password": "987"
                    },
                    "revision": 0,
                    "shared_folder": {
                        "read": {
                            "url": "https://nextcloud.example.com/s/TArrLyC3K7c5Jbg",
                            "password": "DLgoYrFEoy"
                        },
                    },
                    "adhoc_retention_seconds" : 86400,
                    "streaming_targets": Vec::<RoomStreamingTarget>::new(),
                },
                "invitee": {
                    "email": "lastname@example.org",
                    "title": "Prof. Dr.",
                    "first_name": "FirstName",
                    "last_name": "LastName",
                    "language": "de"
                },
                "inviter": {
                    "email": "bob@example.org",
                    "title": "Prof. Dr.",
                    "first_name": "Bob",
                    "last_name": "Inviter",
                    "language": "de"
                }
            }))
            .unwrap()
        );
    }

    #[test]
    fn test_no_time() {
        let basic_invite = MailTask::V1(Message::RegisteredEventInvite(RegisteredEventInvite {
            inviter: RegisteredUser {
                email: "bob@example.org".into(),
                title: "Prof. Dr.".parse().expect("valid user title"),
                first_name: "Bob".into(),
                last_name: "Inviter".into(),
                language: "de".parse().expect("valid language"),
            },
            event: Event {
                id: Uuid::from_u128(1),
                name: "Guten Morgen Meeting".parse().expect("valid event title"),
                description: "".parse().expect("valid event description"),
                created_at: Time {
                    time: chrono::DateTime::<FixedOffset>::parse_from_rfc3339(
                        "2021-12-29T15:00:00+00:00",
                    )
                    .unwrap()
                    .into(),
                    timezone: "UTC".into(),
                },
                start_time: None,
                end_time: None,
                rrule: None,
                room: Room {
                    id: Uuid::from_u128(0),
                    alias: None,
                    password: None,
                },
                call_in: Some(CallIn {
                    sip_tel: "+497652917".into(),
                    sip_id: "2".into(),
                    sip_password: "987".into(),
                }),
                revision: 0,
                shared_folder: Some(SharedFolder {
                    read: SharedFolderAccess {
                        url: "https://nextcloud.example.com/s/TArrLyC3K7c5Jbg".to_string(),
                        password: "DLgoYrFEoy".to_string(),
                    },
                    read_write: None,
                }),
                adhoc_retention_seconds: None,
                streaming_targets: Vec::new(),
            },
            invitee: RegisteredUser {
                email: "lastname@example.org".into(),
                title: "Prof. Dr.".parse().expect("valid user title"),
                first_name: "FirstName".into(),
                last_name: "LastName".into(),
                language: "de".parse().expect("valid language"),
            },
        }));

        assert_eq!(
            basic_invite,
            serde_json::from_value(serde_json::json!({
                "version": "1",
                "message": "registered_event_invite",
                "event": {
                    "id": Uuid::from_u128(1),
                    "name": "Guten Morgen Meeting",
                    "created_at": {"time":"2021-12-29T15:00:00+00:00", "timezone": "UTC"},
                    "description": "",
                    "room": {
                        "id": Uuid::from_u128(0),
                    },
                    "call_in": {
                        "sip_tel": "+497652917",
                        "sip_id": "2",
                        "sip_password": "987"
                    },
                    "revision": 0,
                    "shared_folder": {
                        "read": {
                            "url": "https://nextcloud.example.com/s/TArrLyC3K7c5Jbg",
                            "password": "DLgoYrFEoy"
                        },
                    },
                    "streaming_targets": Vec::<RoomStreamingTarget>::new(),
                },
                "invitee": {
                    "email": "lastname@example.org",
                    "title": "Prof. Dr.",
                    "first_name": "FirstName",
                    "last_name": "LastName",
                    "language": "de"
                },
                "inviter": {
                    "email": "bob@example.org",
                    "title": "Prof. Dr.",
                    "first_name": "Bob",
                    "last_name": "Inviter",
                    "language": "de"
                }
            }))
            .unwrap()
        );
    }
}

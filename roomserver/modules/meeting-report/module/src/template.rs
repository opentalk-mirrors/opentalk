// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types used inside the tera template.
// IMPORTANT: when changing the structs below, make sure to update the following documentation:
// * docs/admin/core/meeting_reports.md

use icu_locid::LanguageIdentifier;
use opentalk_report_generation::ReportDateTime;
use opentalk_types_common::{
    events::{EventDescription, EventTitle},
    time::TimeZone,
};
use opentalk_types_signaling::{ParticipantId, Role};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReportTemplateParameter {
    pub available_languages: Vec<LanguageIdentifier>,

    pub title: EventTitle,

    pub description: EventDescription,

    /// The start date and time of the event, in the local timezone as indicated by `starts_at_tz`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<ReportDateTime>,

    /// The end date and time of the event, in the local timezone as indicated by `ends_at_tz`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<ReportDateTime>,

    /// The date and time at which the report was created.
    pub report_created_at: ReportDateTime,

    /// The timezone in which the timestamps in this report are represented.
    pub report_timezone: TimeZone,

    /// The language in which the report should be created.
    pub report_language: LanguageIdentifier,

    /// The participants in the meeting.
    pub participants: Vec<ReportParticipant>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportParticipant {
    pub id: ParticipantId,

    pub name: String,

    pub role: Role,

    pub email: String,

    pub joined_at: ReportDateTime,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_at: Option<ReportDateTime>,
}

#[cfg(test)]
pub(crate) mod tests {
    use icu_locid::langid;
    use opentalk_types_signaling::{ParticipantId, Role};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::{ReportParticipant, ReportTemplateParameter};
    use crate::AVAILABLE_LANGUAGES;

    pub fn example_small() -> ReportTemplateParameter {
        ReportTemplateParameter {
            available_languages: AVAILABLE_LANGUAGES.to_vec(),
            title: "Testmeeting"
                .parse()
                .expect("value must be parsable as EventTitle"),
            description: ""
                .parse()
                .expect("value must be parsable as EventDescription"),
            starts_at: None,
            ends_at: None,
            report_created_at: "2025-02-06T09:16:47"
                .parse()
                .expect("value must be parsable as ReportDateTime"),
            report_timezone: "Europe/Berlin"
                .parse()
                .expect("value must be parsable as TimeZone"),
            report_language: langid!("en"),
            participants: vec![ReportParticipant {
                id: ParticipantId::from_u128(0x263e256d_adf8_4548_bf77_9262959cd124),
                name: "Alice Adams".into(),
                role: Role::Moderator,
                email: "".into(),
                joined_at: "2025-02-06T08:18:23"
                    .parse()
                    .expect("value must be parsable as ReportDateTime"),
                left_at: None,
            }],
        }
    }

    fn example_small_json() -> serde_json::Value {
        json!({
            "available_languages": ["en", "de"],
            "title": "Testmeeting",
            "description": "",
            "report_created_at": "2025-02-06T09:16:47",
            "report_timezone": "Europe/Berlin",
            "report_language": "en",
            "participants": [
                {
                    "id":"263e256d-adf8-4548-bf77-9262959cd124",
                    "name": "Alice Adams",
                    "role": "moderator",
                    "joined_at": "2025-02-06T08:18:23",
                    "email": "",
                },
            ],
        })
    }

    pub fn example_medium() -> ReportTemplateParameter {
        ReportTemplateParameter {
            available_languages: AVAILABLE_LANGUAGES.to_vec(),
            title: "Testmeeting"
                .parse()
                .expect("value must be parsable as EventTitle"),
            description: "A medium sized test meeting"
                .parse()
                .expect("value must be parsable as EventDescription"),
            starts_at: Some(
                "2025-02-06T08:18:23"
                    .parse()
                    .expect("value must be parsable as ReportDateTime"),
            ),
            ends_at: Some(
                "2025-02-06T11:25:00"
                    .parse()
                    .expect("value must be parsable as ReportDateTime"),
            ),
            report_created_at: "2025-02-06T09:16:47"
                .parse()
                .expect("value must be parsable as ReportDateTime"),
            report_timezone: "Europe/Berlin"
                .parse()
                .expect("value must be parsable as Timezone"),
            report_language: langid!("en"),
            participants: vec![
                ReportParticipant {
                    id: ParticipantId::from_u128(0x31acc6f2_dba2_4236_96c7_2c5faf0bda93),
                    name: "Charlie Cooper".into(),
                    role: Role::User,
                    email: "charlie@example.com".into(),
                    joined_at: "2025-02-06T08:16:30"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                    left_at: Some(
                        "2025-02-06T08:18:12"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    ),
                },
                ReportParticipant {
                    id: ParticipantId::from_u128(0x31acc6f2_dba2_4236_96c7_2c5faf0bda93),
                    name: "Bob Burton".into(),
                    role: Role::User,
                    email: "bob@example.com".into(),
                    joined_at: "2025-02-06T08:16:03"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                    left_at: None,
                },
                ReportParticipant {
                    id: ParticipantId::from_u128(0x263e256d_adf8_4548_bf77_9262959cd124),
                    name: "Alice Adams".into(),
                    role: Role::Moderator,
                    email: "".into(),
                    joined_at: "2025-02-06T08:16:03"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                    left_at: None,
                },
            ],
        }
    }

    fn example_medium_json() -> serde_json::Value {
        json!({
            "available_languages": ["en", "de"],
            "title": "Testmeeting",
            "description": "A medium sized test meeting",
            "starts_at": "2025-02-06T08:18:23",
            "ends_at": "2025-02-06T11:25:00",
            "report_created_at": "2025-02-06T09:16:47",
            "report_timezone": "Europe/Berlin",
            "report_language": "en",
            "participants": [
                {
                    "id":"31acc6f2-dba2-4236-96c7-2c5faf0bda93",
                    "name": "Charlie Cooper",
                    "role": "user",
                    "email": "charlie@example.com",
                    "joined_at": "2025-02-06T08:16:30",
                    "left_at": "2025-02-06T08:18:12",
                },
                {
                    "id":"31acc6f2-dba2-4236-96c7-2c5faf0bda93",
                    "name": "Bob Burton",
                    "role": "user",
                    "email": "bob@example.com",
                    "joined_at": "2025-02-06T08:16:03",
                },
                {
                    "id":"263e256d-adf8-4548-bf77-9262959cd124",
                    "name": "Alice Adams",
                    "role": "moderator",
                    "email": "",
                    "joined_at": "2025-02-06T08:16:03",
                },
            ],
        })
    }

    pub fn example_large() -> ReportTemplateParameter {
        ReportTemplateParameter {
            available_languages: AVAILABLE_LANGUAGES.to_vec(),
            title: "Large Testmeeting"
                .parse()
                .expect("value must be parsable as EventTitle"),
            description: "The large test meeting"
                .parse()
                .expect("value must be parsable as EventDescription"),
            starts_at: Some(
                "2025-02-06T08:18:23"
                    .parse()
                    .expect("value must be parsable as ReportDateTime"),
            ),
            ends_at: Some(
                "2025-02-06T11:25:00"
                    .parse()
                    .expect("value must be parsable as ReportDateTime"),
            ),
            report_created_at: "2025-02-06T09:16:47"
                .parse()
                .expect("value must be parsable as ReportDateTime"),
            report_timezone: "Europe/Berlin"
                .parse()
                .expect("value must be parsable as Timezone"),
            report_language: langid!("en"),
            participants: vec![
                ReportParticipant {
                    id: ParticipantId::from_u128(0xe3524b19_503d_4d79_844b_803b1ecd3115),
                    name: "Franz Fischer".into(),
                    role: Role::User,
                    email: "".into(),
                    joined_at: "2025-02-06T08:16:18"
                        .parse()
                        .expect("value must be parsable as Timezone"),
                    left_at: None,
                },
                ReportParticipant {
                    id: ParticipantId::from_u128(0xdd2c831e_c949_4030_b723_3c80da6c8034),
                    name: "Recorder".into(),
                    role: Role::User,
                    email: "".into(),
                    joined_at: "2025-02-06T08:26:20"
                        .parse()
                        .expect("value must be parsable as Timezone"),
                    left_at: None,
                },
                ReportParticipant {
                    id: ParticipantId::from_u128(0x855c575d_b48e_4463_8b63_8f193d556867),
                    name: "Erin".into(),
                    role: Role::Guest,
                    email: "".into(),
                    joined_at: "2025-02-06T08:16:50"
                        .parse()
                        .expect("value must be parsable as Timezone"),
                    left_at: None,
                },
                ReportParticipant {
                    id: ParticipantId::from_u128(0x97d10184_2080_4807_87f2_2de07eb05948),
                    name: "Dave Dunn".into(),
                    role: Role::Guest,
                    email: "".into(),
                    joined_at: "2025-02-06T08:16:40"
                        .parse()
                        .expect("value must be parsable as Timezone"),
                    left_at: None,
                },
                ReportParticipant {
                    id: ParticipantId::from_u128(0x31acc6f2_dba2_4236_96c7_2c5faf0bda93),
                    name: "Charlie Cooper".into(),
                    role: Role::User,
                    email: "charlie@example.com".into(),
                    joined_at: "2025-02-06T08:16:30"
                        .parse()
                        .expect("value must be parsable as Timezone"),
                    left_at: None,
                },
                ReportParticipant {
                    id: ParticipantId::from_u128(0x31acc6f2_dba2_4236_96c7_2c5faf0bda93),
                    name: "Bob Burton".into(),
                    role: Role::User,
                    email: "bob@example.com".into(),
                    joined_at: "2025-02-06T08:16:03"
                        .parse()
                        .expect("value must be parsable as Timezone"),
                    left_at: None,
                },
                ReportParticipant {
                    id: ParticipantId::from_u128(0x263e256d_adf8_4548_bf77_9262959cd124),
                    name: "Alice Adams".into(),
                    role: Role::Moderator,
                    email: "".into(),
                    joined_at: "2025-02-06T08:16:03"
                        .parse()
                        .expect("value must be parsable as Timezone"),
                    left_at: None,
                },
            ],
        }
    }

    fn example_large_json() -> serde_json::Value {
        json!({
            "available_languages": ["en", "de"],
            "title": "Large Testmeeting",
            "description": "The large test meeting",
            "starts_at": "2025-02-06T08:18:23",
            "ends_at": "2025-02-06T11:25:00",
            "report_created_at": "2025-02-06T09:16:47",
            "report_timezone": "Europe/Berlin",
            "report_language": "en",
            "participants": [
                {
                    "id": "e3524b19-503d-4d79-844b-803b1ecd3115",
                    "name": "Franz Fischer",
                    "role": "user",
                    "email": "",
                    "joined_at": "2025-02-06T08:16:18",
                },
                {
                    "id": "dd2c831e-c949-4030-b723-3c80da6c8034",
                    "name": "Recorder",
                    "role": "user",
                    "email": "",
                    "joined_at": "2025-02-06T08:26:20",
                },
                {
                    "id": "855c575d-b48e-4463-8b63-8f193d556867",
                    "name": "Erin",
                    "role": "guest",
                    "email": "",
                    "joined_at": "2025-02-06T08:16:50",
                },
                {
                    "id": "97d10184-2080-4807-87f2-2de07eb05948",
                    "name": "Dave Dunn",
                    "role": "guest",
                    "email": "",
                    "joined_at": "2025-02-06T08:16:40",
                },
                {
                    "id":"31acc6f2-dba2-4236-96c7-2c5faf0bda93",
                    "name": "Charlie Cooper",
                    "role": "user",
                    "email": "charlie@example.com",
                    "joined_at": "2025-02-06T08:16:30",
                },
                {
                    "id":"31acc6f2-dba2-4236-96c7-2c5faf0bda93",
                    "name": "Bob Burton",
                    "role": "user",
                    "email": "bob@example.com",
                    "joined_at": "2025-02-06T08:16:03",
                },
                {
                    "id":"263e256d-adf8-4548-bf77-9262959cd124",
                    "name": "Alice Adams",
                    "role": "moderator",
                    "email": "",
                    "joined_at": "2025-02-06T08:16:03",
                },
            ],
        })
    }

    #[test]
    fn serialize_example_small() {
        assert_eq!(json!(example_small()), example_small_json());
    }

    #[test]
    fn deserialize_example_small() {
        assert_eq!(
            serde_json::from_value::<ReportTemplateParameter>(example_small_json())
                .expect("value must be deserializable"),
            example_small()
        );
    }

    #[test]
    fn serialize_example_medium() {
        assert_eq!(json!(example_medium()), example_medium_json());
    }

    #[test]
    fn deserialize_example_medium() {
        assert_eq!(
            serde_json::from_value::<ReportTemplateParameter>(example_medium_json())
                .expect("value must be deserializable"),
            example_medium()
        );
    }

    #[test]
    fn serialize_example_large() {
        assert_eq!(json!(example_large()), example_large_json());
    }

    #[test]
    fn deserialize_example_large() {
        assert_eq!(
            serde_json::from_value::<ReportTemplateParameter>(example_large_json())
                .expect("value must be deserializable"),
            example_large()
        );
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use icu_locid::LanguageIdentifier;
use serde::{Deserialize, Serialize};

use super::{ResolvedVote, Summary, TimedEvent};

/// The data used to generate a report with typst
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportData {
    pub available_languages: Vec<LanguageIdentifier>,
    pub summary: Summary,
    pub votes: Vec<ResolvedVote>,
    pub events: Vec<TimedEvent>,
    pub report_language: LanguageIdentifier,
}

#[cfg(test)]
pub(crate) mod tests {
    use icu_locid::langid;
    use opentalk_roomserver_types_legal_vote::{
        cancel::{CancelReason, CustomCancelReason},
        issue::{Issue, TechnicalIssueKind},
        tally::Tally,
        user_parameters::Duration,
        vote::{LegalVoteId, VoteOption},
    };
    use opentalk_types_common::users::DisplayName;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::ReportData;
    use crate::{
        protocol::v1::FinalResults,
        report::data::{Event, ResolvedCancel, ResolvedVote, StopReason, Summary, TimedEvent},
    };

    pub(crate) fn example_public() -> ReportData {
        ReportData {
            available_languages: vec![langid!("en"), langid!("de")],
            summary: Summary {
                title: "Weather Vote".into(),
                subtitle: Some("Another one of these weather votes".into()),
                topic: Some("Is the weather good today?".into()),
                pseudonymous: false,
                creator: "Alice Adams"
                    .parse()
                    .expect("value must be parsable as DisplayName"),
                id: LegalVoteId::from_u128(0xee621ab4_72f6_4d39_bbc4_dc1b96a606cf),
                start_time: "2025-01-02T03:04:05"
                    .parse()
                    .expect("value must be parsable as ReportDateTime"),
                participant_count: 8,
                duration: Some(Duration::try_from(300).unwrap()),
                enable_abstain: false,
                auto_close: true,
                end_time: Some(
                    "2025-01-02T03:09:05"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                ),
                stop_reason: StopReason::Auto,
                vote_count: 6,
                final_results: Some(FinalResults::Valid(Tally {
                    yes: 4,
                    no: 2,
                    abstain: Some(5),
                })),
                report_timezone: "Europe/Berlin"
                    .parse()
                    .expect("value must be parsable as Timezone"),
            },
            votes: vec![
                ResolvedVote {
                    name: Some(
                        "Alice Adams"
                            .parse()
                            .expect("value must be parsable as DisplayName"),
                    ),
                    token: "aaaaaaaa".into(),
                    option: VoteOption::Yes,
                    time: Some(
                        "2025-01-02T03:04:24"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    ),
                },
                ResolvedVote {
                    name: Some(
                        "Bob Burton"
                            .parse()
                            .expect("value must be parsable as DisplayName"),
                    ),
                    token: "bbbbbbbb".into(),
                    option: VoteOption::No,
                    time: Some(
                        "2025-01-02T03:04:20"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    ),
                },
                ResolvedVote {
                    name: Some(
                        "Charlie Cooper"
                            .parse()
                            .expect("value must be parsable as DisplayName"),
                    ),
                    token: "cccccccc".into(),
                    option: VoteOption::No,
                    time: Some(
                        "2025-01-02T03:04:21"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    ),
                },
                ResolvedVote {
                    name: Some(
                        "Dave Dunn"
                            .parse()
                            .expect("value must be parsable as DisplayName"),
                    ),
                    token: "dddddddd".into(),
                    option: VoteOption::Yes,
                    time: Some(
                        "2025-01-02T03:04:19"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    ),
                },
                ResolvedVote {
                    name: Some(
                        "Erin Eaton"
                            .parse()
                            .expect("value must be parsable as DisplayName"),
                    ),
                    token: "eeeeeeee".into(),
                    option: VoteOption::Yes,
                    time: Some(
                        "2025-01-02T03:06:00"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    ),
                },
                ResolvedVote {
                    name: Some(
                        "George Grump"
                            .parse()
                            .expect("value must be parsable as DisplayName"),
                    ),
                    token: "gggggggg".into(),
                    option: VoteOption::Yes,
                    time: Some(
                        "2025-01-02T03:06:00"
                            .parse()
                            .expect("value must be parsable as ReportDateTime"),
                    ),
                },
            ],
            events: vec![TimedEvent {
                time: Some(
                    "2025-01-02T03:04:18"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                ),
                event: Event::Issue {
                    name: Some(
                        "Charlie Cooper"
                            .parse()
                            .expect("value must be parsable as DisplayName"),
                    ),
                    issue: Issue::Technical {
                        kind: TechnicalIssueKind::Screenshare,
                        description: None,
                    },
                },
            }],
            report_language: langid!("en"),
        }
    }

    fn example_public_json() -> serde_json::Value {
        json!({
            "available_languages": ["en", "de"],
            "summary": {
                "title": "Weather Vote",
                "subtitle": "Another one of these weather votes",
                "topic": "Is the weather good today?",
                "pseudonymous": false,
                "creator": "Alice Adams",
                "id": "ee621ab4-72f6-4d39-bbc4-dc1b96a606cf",
                "start_time": "2025-01-02T03:04:05",
                "participant_count": 8,
                "duration": 300,
                "enable_abstain": false,
                "auto_close": true,
                "end_time": "2025-01-02T03:09:05",
                "stop_reason": {
                    "kind": "auto",
                },
                "vote_count": 6,
                "final_results": {
                    "results": "valid",
                    "yes": 4,
                    "no": 2,
                    "abstain": 5,
                },
                "report_timezone": "Europe/Berlin"
            },
            "votes": [
                {
                    "name": "Alice Adams",
                    "token": "aaaaaaaa",
                    "option": "yes",
                    "time": "2025-01-02T03:04:24"
                },
                {
                    "name": "Bob Burton",
                    "token": "bbbbbbbb",
                    "option": "no",
                    "time": "2025-01-02T03:04:20",
                },
                {
                    "name": "Charlie Cooper",
                    "token": "cccccccc",
                    "option": "no",
                    "time": "2025-01-02T03:04:21",
                },
                {
                    "name": "Dave Dunn",
                    "token": "dddddddd",
                    "option": "yes",
                    "time": "2025-01-02T03:04:19",
                },
                {
                    "name": "Erin Eaton",
                    "token": "eeeeeeee",
                    "option": "yes",
                    "time": "2025-01-02T03:06:00",
                },
                {
                    "name": "George Grump",
                    "token": "gggggggg",
                    "option": "yes",
                    "time": "2025-01-02T03:06:00",
                },

            ],
            "events": [
                {
                    "kind": "issue",
                    "event_details": {
                        "name": "Charlie Cooper",
                        "kind": "screenshare",
                    },
                    "time": "2025-01-02T03:04:18",
                },
            ],
            "report_language": "en",
        })
    }

    #[test]
    fn serialize_report_data() {
        assert_eq!(json!(example_public()), example_public_json());
    }

    #[test]
    fn deserialize_report_data() {
        assert_eq!(
            serde_json::from_value::<ReportData>(example_public_json())
                .expect("value must be deserializable"),
            example_public(),
        );
    }

    pub(crate) fn canceled_public() -> ReportData {
        let mut report_data = example_public();
        report_data.summary.final_results = None;
        report_data.summary.stop_reason = StopReason::Canceled(ResolvedCancel {
            user: DisplayName::from_str_lossy("Bob"),
            reason: CancelReason::Custom(CustomCancelReason::try_from("test").unwrap()),
        });

        report_data
    }

    pub(crate) fn initiator_left_public() -> ReportData {
        let mut report_data = example_public();
        report_data.summary.final_results = None;
        report_data.summary.stop_reason = StopReason::Canceled(ResolvedCancel {
            user: DisplayName::from_str_lossy("Alice Aal"),
            reason: CancelReason::InitiatorLeft,
        });

        report_data
    }

    pub(crate) fn room_destroyed_public() -> ReportData {
        let mut report_data = example_public();
        report_data.summary.final_results = None;
        report_data.summary.stop_reason = StopReason::Canceled(ResolvedCancel {
            user: DisplayName::from_str_lossy("Alice Aal"),
            reason: CancelReason::RoomDestroyed,
        });

        report_data
    }

    pub(crate) fn example_pseudonymous() -> ReportData {
        ReportData {
            available_languages: vec![langid!("en"), langid!("de")],
            summary: Summary {
                title: "Example Pseudonymous Vote".into(),
                subtitle: None,
                topic: None,
                pseudonymous: true,
                creator: "Alice Adams"
                    .parse()
                    .expect("value must be parsable as DisplayName"),
                id: LegalVoteId::from_u128(0x6a3525fc_aeef_4d7e_9d76_e41ab2cbe469),
                start_time: "2025-02-08T12:32:09"
                    .parse()
                    .expect("value must be parsable as ReportDateTime"),
                participant_count: 4,
                duration: Some(Duration::try_from(60).unwrap()),
                enable_abstain: true,
                auto_close: true,
                end_time: Some(
                    "2025-02-08T12:32:22"
                        .parse()
                        .expect("value must be parsable as ReportDateTime"),
                ),
                stop_reason: StopReason::Auto,
                vote_count: 4,
                final_results: Some(FinalResults::Valid(Tally {
                    yes: 1,
                    no: 2,
                    abstain: Some(1),
                })),
                report_timezone: "Europe/Vienna"
                    .parse()
                    .expect("value must be parsable as Timezone"),
            },
            votes: vec![
                ResolvedVote {
                    name: None,
                    token: "LPwNXJWs7b1".into(),
                    option: VoteOption::Yes,
                    time: None,
                },
                ResolvedVote {
                    name: None,
                    token: "K5SMSt98f11".into(),
                    option: VoteOption::No,
                    time: None,
                },
                ResolvedVote {
                    name: None,
                    token: "B1yWM5eWQQi".into(),
                    option: VoteOption::Abstain,
                    time: None,
                },
                ResolvedVote {
                    name: None,
                    token: "8PCkuJ9NGoY".into(),
                    option: VoteOption::No,
                    time: None,
                },
            ],
            events: vec![],
            report_language: langid!("en"),
        }
    }
}

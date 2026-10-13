// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use icu_locid::{LanguageIdentifier, subtags::Language};
use opentalk_report_generation::ReportDateTime;
use opentalk_types_common::events::EventTitle;
use serde::Serialize;

use crate::state::protocol::{ProtocolEntry, Speaker};

#[derive(Debug, Serialize)]
pub struct ReportTemplateParameter {
    pub available_languages: Vec<LanguageIdentifier>,
    /// The language in which the report should be generated.
    pub report_language: LanguageIdentifier,
    /// The language the participants were speaking.
    pub transcription_language: Option<Language>,
    /// The title of the event for which the report is generated.
    pub title: Option<EventTitle>,
    /// The time at which the protocol started.
    pub started_at: ReportDateTime,
    /// The time at which the protocol ended.
    pub ended_at: ReportDateTime,
    /// The number of participants that were present at any time while the protocol was ongoing.
    pub present_participant_count: usize,
    /// The moderators that were present while the protocol was ongoing.
    pub moderators: Vec<Speaker>,
    /// Whether or not participant names are replaced with pseudonyms.
    pub pseudonymous: bool,
    /// The recorded entries of the meeting protocol.
    pub protocol: Vec<ProtocolEntry>,
}

#[cfg(test)]
mod tests {
    use icu_locid::langid;
    use insta::assert_snapshot;
    use opentalk_report_generation::ToReportDateTime as _;
    use opentalk_types_common::{
        time::{TimeZone, Timestamp},
        users::DisplayName,
    };

    use super::*;
    use crate::state::protocol::{ProtocolEvent, Speaker};

    #[test]
    fn serialize_report_template_parameter() {
        let timestamp = Timestamp::unix_epoch().to_report_date_time(&TimeZone::utc());
        let parameter = ReportTemplateParameter {
            available_languages: vec![langid!("en"), langid!("de")],
            report_language: langid!("en"),
            transcription_language: None,
            title: Some(EventTitle::from_str_lossy("Title")),
            started_at: timestamp,
            ended_at: timestamp,
            present_participant_count: 3,
            moderators: vec![Speaker::Pseudonym { number: 1 }],
            pseudonymous: true,
            protocol: vec![
                ProtocolEntry {
                    timestamp,
                    event: ProtocolEvent::Start,
                },
                ProtocolEntry {
                    timestamp: Timestamp::unix_epoch().to_report_date_time(&TimeZone::utc()),
                    event: ProtocolEvent::Text {
                        speaker: Speaker::Name {
                            name: DisplayName::from_str_lossy("Alice"),
                        },
                        text: "Hello world".to_string(),
                    },
                },
            ],
        };
        let raw = serde_json::to_string_pretty(&parameter).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "available_languages": [
            "en",
            "de"
          ],
          "report_language": "en",
          "transcription_language": null,
          "title": "Title",
          "started_at": "1970-01-01T00:00:00",
          "ended_at": "1970-01-01T00:00:00",
          "present_participant_count": 3,
          "moderators": [
            {
              "kind": "pseudonym",
              "number": 1
            }
          ],
          "pseudonymous": true,
          "protocol": [
            {
              "timestamp": "1970-01-01T00:00:00",
              "event": {
                "kind": "start"
              }
            },
            {
              "timestamp": "1970-01-01T00:00:00",
              "event": {
                "kind": "text",
                "speaker": {
                  "kind": "name",
                  "name": "Alice"
                },
                "text": "Hello world"
              }
            }
          ]
        }
        "#);
    }
}

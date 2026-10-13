// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `transcription` namespace

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use opentalk_types_common::time::TimeZone;
use serde::{Deserialize, Serialize};

use crate::{event::TranscriptionEvent, service::event::TranscriptionServiceEvent};

/// Commands for the `transcription` namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum TranscriptionCommand {
    /// Start live transcription
    Start {
        /// Optional language hint for the transcription service, e.g. "en" or "de".
        #[serde(skip_serializing_if = "Option::is_none")]
        language: Option<String>,
    },

    /// Stop live transcription.
    Stop,

    ChangeLanguage {
        language: Option<String>,
    },

    /// Start the meeting protocol
    StartMeetingProtocol {
        #[serde(skip_serializing_if = "Option::is_none")]
        /// Optional language hint for the transcription service, e.g. "en" or "de".
        ///
        /// Only honored if the transcription service is not already running, e.g. due to a
        /// previous [`Start`](TranscriptionCommand::Start) command.
        language: Option<String>,

        /// An optional timezone for the PDF generation.
        ///
        /// Falls back to the participants time zone, then UTC.
        /// The timezone should be in a format standardized by IANA (e.g., "CET" or
        /// "Europe/Vienna"). For more details, visit: <https://www.iana.org/time-zones>.
        #[serde(skip_serializing_if = "Option::is_none")]
        timezone: Option<TimeZone>,

        /// Replace participant names with pseudonyms in the meeting protocol.
        pseudonymous: bool,
    },

    /// Pause the meeting protocol
    PauseMeetingProtocol,

    /// Resume the meeting protocol.
    ResumeMeetingProtocol,

    /// Stop the meeting protocol.
    StopMeetingProtocol,

    /// Set the participants consent status to be recorded in the meeting protocol
    SetMeetingProtocolConsent {
        /// Flag indicating whether the participant consents to being recorded
        consent: bool,
    },

    TranscriptionServiceEvent {
        event: TranscriptionServiceEvent,
    },
}

impl CreateReplica<TranscriptionEvent> for TranscriptionCommand {
    fn replicate(&self) -> Option<TranscriptionEvent> {
        None
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
    use opentalk_types_common::{time::Timestamp, transcription::RequestId};
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_start_command() {
        let command = TranscriptionCommand::Start {
            language: Some("de".into()),
        };

        assert_json_snapshot!(command, @ r#"
        {
          "action": "start",
          "language": "de"
        }
        "#);
    }

    #[test]
    fn serialize_start_command_without_language() {
        let command = TranscriptionCommand::Start { language: None };

        assert_json_snapshot!(command, @ r#"
        {
          "action": "start"
        }
        "#);
    }

    #[test]
    fn serialize_stop_command() {
        let command = TranscriptionCommand::Stop;

        assert_json_snapshot!(command, @ r#"
        {
          "action": "stop"
        }
        "#);
    }

    #[test]
    fn serialize_start_meeting_protocol() {
        let command = TranscriptionCommand::StartMeetingProtocol {
            language: None,
            timezone: None,
            pseudonymous: false,
        };

        assert_json_snapshot!(command, @r#"
        {
          "action": "start_meeting_protocol",
          "pseudonymous": false
        }
        "#);
    }

    #[test]
    fn deserialize_start_meeting_protocol() {
        let expected = TranscriptionCommand::StartMeetingProtocol {
            language: None,
            timezone: None,
            pseudonymous: false,
        };
        let produced = serde_json::from_value(json!({
           "action": "start_meeting_protocol",
           "pseudonymous": false,
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_pause_meeting_protocol() {
        let command = TranscriptionCommand::PauseMeetingProtocol;

        assert_json_snapshot!(command, @r#"
        {
          "action": "pause_meeting_protocol"
        }
        "#);
    }

    #[test]
    fn deserialize_pause_meeting_protocol() {
        let expected = TranscriptionCommand::PauseMeetingProtocol;
        let produced = serde_json::from_value(json!({
           "action": "pause_meeting_protocol"
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_resume_meeting_protocol() {
        let command = TranscriptionCommand::ResumeMeetingProtocol;

        assert_json_snapshot!(command, @r#"
        {
          "action": "resume_meeting_protocol"
        }
        "#);
    }

    #[test]
    fn deserialize_resume_meeting_protocol() {
        let expected = TranscriptionCommand::ResumeMeetingProtocol;
        let produced = serde_json::from_value(json!({
           "action": "resume_meeting_protocol"
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_stop_meeting_protocol() {
        let command = TranscriptionCommand::StopMeetingProtocol;

        assert_json_snapshot!(command, @r#"
        {
          "action": "stop_meeting_protocol"
        }
        "#);
    }

    #[test]
    fn deserialize_stop_meeting_protocol() {
        let expected = TranscriptionCommand::StopMeetingProtocol;
        let produced = serde_json::from_value(json!({
           "action": "stop_meeting_protocol"
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_set_meeting_protocol_consent() {
        let command = TranscriptionCommand::SetMeetingProtocolConsent { consent: true };

        assert_json_snapshot!(command, @r#"
        {
          "action": "set_meeting_protocol_consent",
          "consent": true
        }
        "#);
    }

    #[test]
    fn deserialize_set_meeting_protocol_consent() {
        let expected = TranscriptionCommand::SetMeetingProtocolConsent { consent: true };
        let produced = serde_json::from_value(json!({
           "action": "set_meeting_protocol_consent",
           "consent": true,
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_service_event_started() {
        let command = TranscriptionCommand::TranscriptionServiceEvent {
            event: TranscriptionServiceEvent::Started {
                request_id: RequestId::nil(),
                language: Some("de".into()),
            },
        };

        assert_json_snapshot!(command, @ r#"
        {
          "action": "transcription_service_event",
          "event": {
            "kind": "started",
            "request_id": "00000000-0000-0000-0000-000000000000",
            "language": "de"
          }
        }
        "#);
    }

    #[test]
    fn serialize_service_event_segment() {
        let segment = crate::segment::TranscriptionSegment {
            participant_id: ParticipantId::nil(),
            track_id: "track1".into(),
            starts_at: Timestamp::unix_epoch(),
            ends_at: Timestamp::unix_epoch(),
            text: "Hello world".into(),
        };

        let command = TranscriptionCommand::TranscriptionServiceEvent {
            event: TranscriptionServiceEvent::Segment(segment.clone()),
        };

        assert_json_snapshot!(command, @ r#"
        {
          "action": "transcription_service_event",
          "event": {
            "kind": "segment",
            "participant_id": "00000000-0000-0000-0000-000000000000",
            "track_id": "track1",
            "starts_at": "1970-01-01T00:00:00Z",
            "ends_at": "1970-01-01T00:00:00Z",
            "text": "Hello world"
          }
        }
        "#);
    }
}

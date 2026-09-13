// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `transcription` namespace

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use serde::{Deserialize, Serialize};

use crate::{event::TranscriptionEvent, service::event::TranscriptionServiceEvent};

/// Commands for the `transcription` namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum TranscriptionCommand {
    Start {
        /// Optional language hint for the transcription service, e.g. "en" or "de".
        #[serde(skip_serializing_if = "Option::is_none")]
        language: Option<String>,
    },

    Stop,

    ChangeLanguage {
        language: Option<String>,
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
    use opentalk_types_common::time::Timestamp;
    use opentalk_types_signaling::ParticipantId;

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
    fn serialize_service_event_started() {
        let command = TranscriptionCommand::TranscriptionServiceEvent {
            event: TranscriptionServiceEvent::Started {
                language: Some("de".into()),
            },
        };

        assert_json_snapshot!(command, @ r#"
        {
          "action": "transcription_service_event",
          "event": {
            "kind": "started",
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

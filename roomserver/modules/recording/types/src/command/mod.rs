// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `recording` namespace

use std::collections::BTreeSet;

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use opentalk_types_common::streaming::StreamingTargetId;
use serde::{Deserialize, Serialize};

use crate::{event::RecordingEvent, service::event::RecordingServiceEvent};

/// Commands for the `recording` namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum RecordingCommand {
    /// Set the participants consent status for all recordings and streams in the conference
    SetConsent {
        /// Flag indicating whether the participant consents to being recorded
        consent: bool,
    },

    /// Starts recording in the current room
    StartRecording,

    /// Pauses recording in the current room
    PauseRecording,

    /// Stop recording in the current room
    StopRecording,

    /// Starts a streaming target
    StartStream {
        target_ids: BTreeSet<StreamingTargetId>,
    },

    /// Pauses a streaming target
    PauseStream {
        target_ids: BTreeSet<StreamingTargetId>,
    },

    /// Stops a streaming target
    StopStream {
        target_ids: BTreeSet<StreamingTargetId>,
    },

    /// Service command, can only be sent by recorders
    Service { command: RecordingServiceEvent },
}

impl CreateReplica<RecordingEvent> for RecordingCommand {
    fn replicate(&self) -> Option<RecordingEvent> {
        None
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::RecordingStatus;

    #[test]
    fn serialize_set_consent() {
        let command = RecordingCommand::SetConsent { consent: true };

        assert_json_snapshot!(command, @ r#"
        {
          "action": "set_consent",
          "consent": true
        }
        "#);
    }

    #[test]
    fn deserialize_set_consent() {
        let json = json!(
            {
                "action": "set_consent",
                "consent": true
            }
        );

        let produced: RecordingCommand = serde_json::from_value(json).unwrap();
        let expected = RecordingCommand::SetConsent { consent: true };

        assert_eq!(produced, expected)
    }

    #[test]
    fn serialize_start_recording() {
        let command = RecordingCommand::StartRecording;

        assert_json_snapshot!(command, @ r#"
        {
          "action": "start_recording"
        }
        "#);
    }

    #[test]
    fn deserialize_start_recording() {
        let json = json!(
            {
                "action": "start_recording"
            }
        );

        let produced: RecordingCommand = serde_json::from_value(json).unwrap();
        let expected = RecordingCommand::StartRecording;

        assert_eq!(produced, expected)
    }

    #[test]
    fn serialize_start_stream() {
        let command = RecordingCommand::StartStream {
            target_ids: [
                StreamingTargetId::from_u128(1),
                StreamingTargetId::from_u128(2),
                StreamingTargetId::from_u128(3),
            ]
            .into(),
        };

        assert_json_snapshot!(command, @ r#"
        {
          "action": "start_stream",
          "target_ids": [
            "00000000-0000-0000-0000-000000000001",
            "00000000-0000-0000-0000-000000000002",
            "00000000-0000-0000-0000-000000000003"
          ]
        }
        "#);
    }

    #[test]
    fn deserialize_start_stream() {
        let json = json!(
            {
                "action": "start_stream",
                "target_ids": [
                    "00000000-0000-0000-0000-000000000001",
                    "00000000-0000-0000-0000-000000000002",
                    "00000000-0000-0000-0000-000000000003"
                ]
            }
        );

        let produced: RecordingCommand = serde_json::from_value(json).unwrap();
        let expected = RecordingCommand::StartStream {
            target_ids: [
                StreamingTargetId::from_u128(1),
                StreamingTargetId::from_u128(2),
                StreamingTargetId::from_u128(3),
            ]
            .into(),
        };

        assert_eq!(produced, expected)
    }

    #[test]
    fn serialize_service_command() {
        let command = RecordingCommand::Service {
            command: RecordingServiceEvent::RecordingUpdated(RecordingStatus::Active),
        };

        assert_json_snapshot!(command, @ r#"
        {
          "action": "service",
          "command": {
            "kind": "recording_updated",
            "status": "active"
          }
        }
        "#);
    }

    #[test]
    fn deserialize_service_command() {
        let json = json!(
            {
                "action": "service",
                "command": {
                    "kind": "recording_updated",
                    "status": "active"
                }
            }
        );

        let produced: RecordingCommand = serde_json::from_value(json).unwrap();
        let expected = RecordingCommand::Service {
            command: RecordingServiceEvent::RecordingUpdated(RecordingStatus::Active),
        };

        assert_eq!(produced, expected)
    }
}

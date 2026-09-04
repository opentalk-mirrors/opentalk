// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to the state of the `transcription` module. This is the state that is sent to a
//! participant when they join the meeting.

use serde::{Deserialize, Serialize};

use crate::{config::MeetingProtocolConfig, status::TranscriptionStatus};

/// The state of the `transcription` module.
///
/// This struct is sent to the participant in the `join_success` message
/// when they successfully join the meeting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranscriptionState {
    /// The status of the transcription service
    #[serde(flatten)]
    pub status: TranscriptionStatus,
    /// If `true` live transcription is enabled.
    pub live_transcription: bool,
    /// If `Some`, a meeting protocol is recorded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meeting_protocol: Option<MeetingProtocolState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeetingProtocolState {
    #[serde(flatten)]
    pub config: MeetingProtocolConfig,
    pub consents: bool,
    pub paused: bool,
}

impl opentalk_types_signaling::SignalingModuleFrontendData for TranscriptionState {
    const NAMESPACE: Option<opentalk_types_common::modules::ModuleId> =
        Some(crate::TRANSCRIPTION_MODULE_ID);
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_inactive_state() {
        let state = TranscriptionState {
            status: TranscriptionStatus::Inactive,
            live_transcription: false,
            meeting_protocol: None,
        };

        assert_json_snapshot!(state, @ r#"
        {
          "status": "inactive",
          "live_transcription": false
        }
        "#);
    }

    #[test]
    fn serialize_requested_state() {
        let state = TranscriptionState {
            status: TranscriptionStatus::Requested {
                language: Some("de".into()),
            },
            live_transcription: true,
            meeting_protocol: None,
        };

        assert_json_snapshot!(state, @ r#"
        {
          "status": "requested",
          "language": "de",
          "live_transcription": true
        }
        "#);
    }

    #[test]
    fn serialize_running_state() {
        let state = TranscriptionState {
            status: TranscriptionStatus::Running {
                language: Some("en".into()),
            },
            live_transcription: false,
            meeting_protocol: Some(MeetingProtocolState {
                config: MeetingProtocolConfig { pseudonymous: true },
                consents: false,
                paused: true,
            }),
        };

        assert_json_snapshot!(state, @ r#"
        {
          "status": "running",
          "language": "en",
          "live_transcription": false,
          "meeting_protocol": {
            "pseudonymous": true,
            "consents": false,
            "paused": true
          }
        }
        "#);
    }

    #[test]
    fn serialize_meeting_protocol_state() {
        let state = MeetingProtocolState {
            config: MeetingProtocolConfig { pseudonymous: true },
            consents: true,
            paused: false,
        };

        assert_json_snapshot!(state, @ r#"
        {
          "pseudonymous": true,
          "consents": true,
          "paused": false
        }
        "#);
    }

    #[test]
    fn deserialize_meeting_protocol_state() {
        let produced = serde_json::from_value(json!({
           "pseudonymous": false,
           "consents": false,
           "paused": true,
        }))
        .unwrap();
        let expected = MeetingProtocolState {
            config: MeetingProtocolConfig {
                pseudonymous: false,
            },
            consents: false,
            paused: true,
        };

        assert_eq!(expected, produced);
    }
}

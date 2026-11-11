// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! Types related to the status of the transcription.
//!
//! The status is split into two independent parts:
//! - [`TranscriptionStatus`] describes the lifecycle of the transcription service (e.g. from
//!   `requested` to `running`).
//! - [`TranscriptionModes`] describes which transcription modes have been activated by the
//!   moderators (live transcription and/or automatic meeting protocol).

use serde::{Deserialize, Serialize};

use crate::{config::AutomaticMeetingProtocolConfig, state::AutomaticMeetingProtocolState};

/// The status of the transcription service
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum TranscriptionStatus {
    /// No transcription service is running
    Inactive,
    /// A transcription service has been requested but has not started yet
    Requested { language: Option<String> },
    /// The transcription service is running
    Running { language: Option<String> },
}

/// The transcription modes activated by the moderators
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranscriptionModes {
    /// If `true` live transcription is enabled.
    pub live_transcription: bool,
    /// If `Some`, an automatic meeting protocol is recorded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub automatic_meeting_protocol: Option<AutomaticMeetingProtocolMode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutomaticMeetingProtocolMode {
    #[serde(flatten)]
    pub config: AutomaticMeetingProtocolConfig,
    /// While paused, no speech is written to the protocol.
    pub paused: bool,
}

impl From<AutomaticMeetingProtocolState> for AutomaticMeetingProtocolMode {
    fn from(value: AutomaticMeetingProtocolState) -> Self {
        Self {
            config: value.config,
            paused: value.paused,
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::{
        config::AutomaticMeetingProtocolConfig,
        state::AutomaticMeetingProtocolState,
        status::{AutomaticMeetingProtocolMode, TranscriptionStatus},
    };

    #[test]
    fn serialize_inactive() {
        let status = TranscriptionStatus::Inactive;

        assert_json_snapshot!(status, @r#"
        {
          "status": "inactive"
        }
        "#);
    }

    #[test]
    fn deserialize_inactive() {
        let expected = TranscriptionStatus::Inactive;
        let produced = serde_json::from_value(json!({
           "status": "inactive"
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_requested() {
        let status = TranscriptionStatus::Requested {
            language: Some("en".to_owned()),
        };

        assert_json_snapshot!(status, @r#"
        {
          "status": "requested",
          "language": "en"
        }
        "#);
    }

    #[test]
    fn deserialize_requested() {
        let expected = TranscriptionStatus::Requested {
            language: Some("en".to_owned()),
        };
        let produced = serde_json::from_value(json!({
           "status": "requested",
           "language": "en",
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_running() {
        let status = TranscriptionStatus::Running {
            language: Some("en".to_owned()),
        };

        assert_json_snapshot!(status, @r#"
        {
          "status": "running",
          "language": "en"
        }
        "#);
    }

    #[test]
    fn deserialize_running() {
        let expected = TranscriptionStatus::Running {
            language: Some("en".to_owned()),
        };
        let produced = serde_json::from_value(json!({
           "status": "running",
           "language": "en",
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_automatic_meeting_protocol_state() {
        let state = AutomaticMeetingProtocolState {
            config: AutomaticMeetingProtocolConfig { pseudonymous: true },
            consents: true,
            paused: false,
        };

        assert_json_snapshot!(state, @r#"
        {
          "pseudonymous": true,
          "consents": true,
          "paused": false
        }
        "#);
    }

    #[test]
    fn deserialize_automatic_meeting_protocol_state() {
        let expected = AutomaticMeetingProtocolState {
            config: AutomaticMeetingProtocolConfig { pseudonymous: true },
            consents: true,
            paused: false,
        };
        let produced = serde_json::from_value(json!({
           "pseudonymous": true,
           "consents": true,
           "paused": false,
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_automatic_meeting_protocol_mode() {
        let mode = AutomaticMeetingProtocolMode {
            config: AutomaticMeetingProtocolConfig { pseudonymous: true },
            paused: false,
        };

        assert_json_snapshot!(mode, @r#"
        {
          "pseudonymous": true,
          "paused": false
        }
        "#);
    }

    #[test]
    fn deserialize_automatic_meeting_protocol_mode() {
        let produced: AutomaticMeetingProtocolMode = serde_json::from_value(json!({
           "pseudonymous": false,
           "paused": true,
        }))
        .unwrap();

        let expected = AutomaticMeetingProtocolMode {
            config: AutomaticMeetingProtocolConfig {
                pseudonymous: false,
            },
            paused: true,
        };

        assert_eq!(expected, produced);
    }
}

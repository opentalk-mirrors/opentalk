// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `transcription` namespace

use opentalk_types_api_internal::module_assets::Quota;
use opentalk_types_common::assets::AssetId;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

mod error;

pub use error::TranscriptionError;

use crate::{
    segment::TranscriptionSegment,
    service::command::TranscriptionServiceCommand,
    status::{TranscriptionModes, TranscriptionStatus},
};

/// Events for the `transcription` namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum TranscriptionEvent {
    Segment(TranscriptionSegment),

    /// The status of the transcription service changed
    StateUpdated {
        #[serde(flatten)]
        status: TranscriptionStatus,
    },

    /// The active transcription modes changed
    ModesUpdated {
        #[serde(flatten)]
        modes: TranscriptionModes,
    },

    /// A PDF asset has been created
    PdfCreated {
        /// The file name of the PDF asset
        filename: String,
        /// The asset id for the PDF asset
        asset_id: AssetId,
        /// The new quota values
        quota: Quota,
    },

    /// A participant updated their automatic transcription consent
    AutomaticTranscriptionConsentUpdated {
        /// The participant that updated their consent
        participant: ParticipantId,
        /// Whether or not the participant consents to being included in the automatic
        /// transcription.
        consents: bool,
    },

    ServiceCommand {
        command: TranscriptionServiceCommand,
    },

    Error(TranscriptionError),
}

impl From<TranscriptionError> for TranscriptionEvent {
    fn from(value: TranscriptionError) -> Self {
        Self::Error(value)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
    use opentalk_types_common::time::Timestamp;
    use opentalk_types_signaling::ParticipantId;
    use serde_json::json;

    use super::*;
    use crate::{config::AutomaticTranscriptionConfig, status::AutomaticTranscriptionMode};

    #[test]
    fn serialize_segment() {
        let event = TranscriptionEvent::Segment(TranscriptionSegment {
            participant_id: ParticipantId::nil(),
            track_id: "track1".into(),
            starts_at: Timestamp::unix_epoch(),
            ends_at: Timestamp::unix_epoch(),
            text: "Hello world".into(),
        });

        assert_json_snapshot!(event, @ r#"
        {
          "message": "segment",
          "participant_id": "00000000-0000-0000-0000-000000000000",
          "track_id": "track1",
          "starts_at": "1970-01-01T00:00:00Z",
          "ends_at": "1970-01-01T00:00:00Z",
          "text": "Hello world"
        }
        "#);
    }

    #[test]
    fn serialize_requested() {
        let event = TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Requested {
                language: Some("de".into()),
            },
        };

        assert_json_snapshot!(event, @ r#"
        {
          "message": "state_updated",
          "status": "requested",
          "language": "de"
        }
        "#);
    }

    #[test]
    fn serialize_running() {
        let event = TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Running {
                language: Some("de".into()),
            },
        };

        assert_json_snapshot!(event, @ r#"
        {
          "message": "state_updated",
          "status": "running",
          "language": "de"
        }
        "#);
    }

    #[test]
    fn serialize_inactive() {
        let event = TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Inactive,
        };

        assert_json_snapshot!(event, @ r#"
        {
          "message": "state_updated",
          "status": "inactive"
        }
        "#);
    }

    #[test]
    fn serialize_modes_updated_live_transcription() {
        let event = TranscriptionEvent::ModesUpdated {
            modes: TranscriptionModes {
                live_transcription: true,
                automatic_transcription: None,
            },
        };

        assert_json_snapshot!(event, @ r#"
        {
          "message": "modes_updated",
          "live_transcription": true
        }
        "#);
    }

    #[test]
    fn deserialize_modes_updated_live_transcription() {
        let produced = serde_json::from_value(json!({
           "message": "modes_updated",
           "live_transcription": true,
        }))
        .unwrap();
        let expected = TranscriptionEvent::ModesUpdated {
            modes: TranscriptionModes {
                live_transcription: true,
                automatic_transcription: None,
            },
        };

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_modes_updated_automatic_transcription() {
        let event = TranscriptionEvent::ModesUpdated {
            modes: TranscriptionModes {
                live_transcription: false,
                automatic_transcription: Some(AutomaticTranscriptionMode {
                    config: AutomaticTranscriptionConfig { pseudonymous: true },
                    paused: false,
                }),
            },
        };

        assert_json_snapshot!(event, @ r#"
        {
          "message": "modes_updated",
          "live_transcription": false,
          "automatic_transcription": {
            "pseudonymous": true,
            "paused": false
          }
        }
        "#);
    }

    #[test]
    fn deserialize_modes_updated_automatic_transcription() {
        let produced = serde_json::from_value(json!({
           "message": "modes_updated",
           "live_transcription": false,
           "automatic_transcription": {
              "pseudonymous": false,
              "paused": false,
           },
        }))
        .unwrap();
        let expected = TranscriptionEvent::ModesUpdated {
            modes: TranscriptionModes {
                live_transcription: false,
                automatic_transcription: Some(AutomaticTranscriptionMode {
                    config: AutomaticTranscriptionConfig {
                        pseudonymous: false,
                    },
                    paused: false,
                }),
            },
        };

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_pdf_created() {
        let event = TranscriptionEvent::PdfCreated {
            filename: "automatic_transcription.pdf".into(),
            asset_id: AssetId::nil(),
            quota: Quota {
                total: Some(1000),
                used: 100,
            },
        };

        assert_json_snapshot!(event, @ r#"
        {
          "message": "pdf_created",
          "filename": "automatic_transcription.pdf",
          "asset_id": "00000000-0000-0000-0000-000000000000",
          "quota": {
            "total": 1000,
            "used": 100
          }
        }
        "#);
    }

    #[test]
    fn deserialize_pdf_created() {
        let produced = serde_json::from_value(json!({
           "message": "pdf_created",
           "filename": "automatic_transcription.pdf",
           "asset_id": "00000000-0000-0000-0000-000000000000",
           "quota": {
              "total": 1000,
              "used": 100,
           },
        }))
        .unwrap();
        let expected = TranscriptionEvent::PdfCreated {
            filename: "automatic_transcription.pdf".into(),
            asset_id: AssetId::nil(),
            quota: Quota {
                total: Some(1000),
                used: 100,
            },
        };

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_automatic_transcription_consent_updated() {
        let event = TranscriptionEvent::AutomaticTranscriptionConsentUpdated {
            participant: ParticipantId::nil(),
            consents: true,
        };

        assert_json_snapshot!(event, @ r#"
        {
          "message": "automatic_transcription_consent_updated",
          "participant": "00000000-0000-0000-0000-000000000000",
          "consents": true
        }
        "#);
    }

    #[test]
    fn deserialize_automatic_transcription_consent_updated() {
        let produced = serde_json::from_value(json!({
           "message": "automatic_transcription_consent_updated",
           "participant": "00000000-0000-0000-0000-000000000000",
           "consents": false,
        }))
        .unwrap();
        let expected = TranscriptionEvent::AutomaticTranscriptionConsentUpdated {
            participant: ParticipantId::nil(),
            consents: false,
        };

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_service_command() {
        let event = TranscriptionEvent::ServiceCommand {
            command: TranscriptionServiceCommand::Stop,
        };

        assert_json_snapshot!(event, @ r#"
        {
          "message": "service_command",
          "command": {
            "kind": "stop"
          }
        }
        "#);
    }

    #[test]
    fn serialize_error() {
        let event = TranscriptionEvent::Error(TranscriptionError::ServiceRequestFailed);

        assert_json_snapshot!(event, @ r#"
        {
          "message": "error",
          "error": "service_request_failed"
        }
        "#);
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `meeting_report` namespace

use opentalk_types_api_internal::module_assets::Quota;
use opentalk_types_common::{assets::AssetId, time::Timestamp};
use serde::{Deserialize, Serialize};

use super::TrainingParticipationReportError;
use crate::event::{PresenceLoggingEndedReason, PresenceLoggingStartedReason};

/// Events sent out by the `meeting_report` module
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "message")]
pub enum TrainingParticipationReportEvent {
    /// Information to participants that presence logging has started
    PresenceLoggingStarted {
        /// Timestamp when the first checkpoint starts. Only included in messages sent to the
        /// creator.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        first_checkpoint: Option<Timestamp>,

        /// The reason why presence logging started. Only included in messages sent to the creator.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<PresenceLoggingStartedReason>,
    },

    /// Information to participants that presence logging has ended
    PresenceLoggingEnded {
        /// The reason why presence logging ended.
        reason: PresenceLoggingEndedReason,
    },

    /// Sent to all participants as a request to confirm their presence.
    PresenceConfirmationRequested,

    /// Sent all participants as a confirmation that their presence has been logged.
    PresenceConfirmationLogged,

    /// A PDF asset has been created
    PdfCreated {
        /// The file name of the PDF asset
        filename: String,

        /// The asset id for the PDF asset
        asset_id: AssetId,

        /// The new quota values
        quota: Quota,
    },

    /// An error happened when executing a `meeting_report` command
    Error(TrainingParticipationReportError),
}

impl From<TrainingParticipationReportError> for TrainingParticipationReportEvent {
    fn from(value: TrainingParticipationReportError) -> Self {
        Self::Error(value)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_types_api_internal::module_assets::Quota;
    use opentalk_types_common::{assets::AssetId, time::Timestamp, utils::ExampleData};
    use serde_json::json;

    use super::{TrainingParticipationReportError, TrainingParticipationReportEvent};
    use crate::event::PresenceLoggingStartedReason;

    #[test]
    fn serialize_presence_logging_started() {
        let event = TrainingParticipationReportEvent::PresenceLoggingStarted {
            first_checkpoint: None,
            reason: None,
        };
        let raw = serde_json::to_string_pretty(&event).expect("Must be serializable");
        assert_snapshot!(raw, @r#"
        {
          "message": "presence_logging_started"
        }
        "#);
    }

    #[test]
    fn deserialize_presence_logging_started() {
        let json = json!({
            "message": "presence_logging_started",
        });
        let event: TrainingParticipationReportEvent =
            serde_json::from_value(json).expect("Must be deserializable");

        assert_eq!(
            event,
            TrainingParticipationReportEvent::PresenceLoggingStarted {
                first_checkpoint: None,
                reason: None,
            }
        );
    }

    #[test]
    fn serialize_presence_logging_started_with_optional_fields() {
        let event = TrainingParticipationReportEvent::PresenceLoggingStarted {
            first_checkpoint: Some(
                "2025-02-03T04:05:06Z"
                    .parse()
                    .expect("must be parseable as timestamp"),
            ),
            reason: Some(PresenceLoggingStartedReason::StartedManually),
        };
        let raw = serde_json::to_string_pretty(&event).expect("Must be serializable");

        assert_snapshot!(raw, @r#"
        {
          "message": "presence_logging_started",
          "first_checkpoint": "2025-02-03T04:05:06Z",
          "reason": "started_manually"
        }
        "#);
    }

    #[test]
    fn deserialize_presence_logging_started_with_optional_fields() {
        let json = json!({
            "message": "presence_logging_started",
            "first_checkpoint": "1970-01-01T00:00:00Z",
            "reason": "started_manually"
        });
        let event: TrainingParticipationReportEvent =
            serde_json::from_value(json).expect("Must be deserializable");

        assert_eq!(
            event,
            TrainingParticipationReportEvent::PresenceLoggingStarted {
                first_checkpoint: Some(Timestamp::unix_epoch()),
                reason: Some(PresenceLoggingStartedReason::StartedManually),
            }
        );
    }

    #[test]
    fn serialize_pdf_created() {
        let event = TrainingParticipationReportEvent::PdfCreated {
            filename: "pdf-file.pdf".to_owned(),
            asset_id: AssetId::from_u128(0x735fcdaa_56dd_4ddb_9eb0_7d083a4a9d9b),
            quota: Quota::example_data(),
        };
        let raw = serde_json::to_string_pretty(&event).expect("Must be serializable");

        assert_snapshot!(raw, @r#"
        {
          "message": "pdf_created",
          "filename": "pdf-file.pdf",
          "asset_id": "735fcdaa-56dd-4ddb-9eb0-7d083a4a9d9b",
          "quota": {
            "total": 5368709120,
            "used": 2147483648
          }
        }
        "#);
    }

    #[test]
    fn deserialize_pdf_created() {
        let json = json!({
            "message": "pdf_created",
            "filename": "pdf-file.pdf",
            "asset_id": "735fcdaa-56dd-4ddb-9eb0-7d083a4a9d9b",
            "quota": {
                "used": 0,
            },
        });
        let event: TrainingParticipationReportEvent =
            serde_json::from_value(json).expect("Must be deserializable");

        assert_eq!(
            event,
            TrainingParticipationReportEvent::PdfCreated {
                filename: "pdf-file.pdf".to_owned(),
                asset_id: AssetId::from_u128(0x735fcdaa_56dd_4ddb_9eb0_7d083a4a9d9b),
                quota: Quota {
                    total: None,
                    used: 0
                }
            }
        );
    }

    #[test]
    fn serialize_error() {
        let pdf_event = TrainingParticipationReportEvent::Error(
            TrainingParticipationReportError::StorageExceeded,
        );
        let raw = serde_json::to_string_pretty(&pdf_event).expect("Must be serializable");

        assert_snapshot!(raw, @r#"
        {
          "message": "error",
          "error": "storage_exceeded"
        }
        "#);
    }

    #[test]
    fn deserialize_error() {
        let json = json!({
            "message": "error",
            "error": "storage_exceeded"
        });
        let event: TrainingParticipationReportEvent =
            serde_json::from_value(json).expect("Must be deserializable");

        assert_eq!(
            event,
            TrainingParticipationReportEvent::Error(
                TrainingParticipationReportError::StorageExceeded
            )
        );
    }
}

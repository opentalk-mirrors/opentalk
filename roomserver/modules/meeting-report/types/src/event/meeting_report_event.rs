// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `meeting_report` namespace

use opentalk_types_common::assets::AssetId;
use serde::{Deserialize, Serialize};

use super::MeetingReportError;

/// Events sent out by the `meeting_report` module
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "message")]
pub enum MeetingReportEvent {
    /// A PDF asset has been created
    PdfAsset {
        /// The file name of the PDF asset
        filename: String,

        /// The asset id for the PDF asset
        asset_id: AssetId,
    },

    ReportGenerationStarted {
        /// Whether or not to the e-mail addresses of the participants were requested
        /// to be included in the report
        include_email_addresses: bool,
    },

    /// An error happened when executing a `meeting_report` command
    Error(MeetingReportError),
}

impl From<MeetingReportError> for MeetingReportEvent {
    fn from(value: MeetingReportError) -> Self {
        Self::Error(value)
    }
}

#[cfg(test)]
mod tests {
    use opentalk_types_common::assets::AssetId;
    use serde_json::json;

    use super::{MeetingReportError, MeetingReportEvent};

    #[test]
    fn serialize_meeting_report_event_pdf_asset() {
        let pdf_event = MeetingReportEvent::PdfAsset {
            filename: "pdf-file.pdf".to_owned(),
            asset_id: AssetId::from_u128(0x735fcdaa_56dd_4ddb_9eb0_7d083a4a9d9b),
        };
        let value = serde_json::to_value(pdf_event).expect("Must be serializable");
        assert_eq!(
            value,
            json!({
                "filename": "pdf-file.pdf",
                "asset_id": "735fcdaa-56dd-4ddb-9eb0-7d083a4a9d9b",
                "message": "pdf_asset",
            })
        );
    }

    #[test]
    fn serialize_meeting_report_event_error() {
        let pdf_event = MeetingReportEvent::Error(MeetingReportError::Generate);
        let value = serde_json::to_value(pdf_event).expect("Must be serializable");
        assert_eq!(
            value,
            json!({
                "error": "generate",
                "message": "error",
            })
        );
    }
}

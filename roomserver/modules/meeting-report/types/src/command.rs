// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `meeting_report` namespace

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use serde::{Deserialize, Serialize};

use crate::event::MeetingReportEvent;

/// Incoming websocket messages
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum MeetingReportCommand {
    /// Generate a report on current and past meeting attendees
    GenerateAttendanceReport {
        /// Whether or not to include the e-mail addresses of the participants in the
        /// report
        include_email_addresses: bool,
    },
}

impl CreateReplica<MeetingReportEvent> for MeetingReportCommand {
    fn replicate(&self) -> Option<MeetingReportEvent> {
        let event = match self {
            MeetingReportCommand::GenerateAttendanceReport {
                include_email_addresses,
            } => MeetingReportEvent::ReportGenerationStarted {
                include_email_addresses: *include_email_addresses,
            },
        };
        Some(event)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::MeetingReportCommand;

    #[test]
    fn create_attendees_report() {
        let json = json!({
            "action": "generate_attendance_report",
            "include_email_addresses": false,
        });

        assert_eq!(
            serde_json::from_value::<MeetingReportCommand>(json).unwrap(),
            MeetingReportCommand::GenerateAttendanceReport {
                include_email_addresses: false
            }
        );
    }
}

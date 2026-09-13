// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use opentalk_types_common::training_participation_report::TimeRange;
use serde::{Deserialize, Serialize};

use crate::TrainingParticipationReportEvent;

/// Incoming websocket messages
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum TrainingParticipationReportCommand {
    /// Enable presence logging
    EnablePresenceLogging {
        /// The time range definition for the initial checkpoint delay.
        #[serde(skip_serializing_if = "Option::is_none")]
        initial_checkpoint_delay: Option<TimeRange>,

        /// The time range definition for the subsequent checkpoints.
        #[serde(skip_serializing_if = "Option::is_none")]
        checkpoint_interval: Option<TimeRange>,
    },

    /// Disable presence logging
    DisablePresenceLogging,

    /// Confirm presence
    ConfirmPresence,
}

impl CreateReplica<TrainingParticipationReportEvent> for TrainingParticipationReportCommand {
    fn replicate(&self) -> Option<TrainingParticipationReportEvent> {
        None
    }
}

#[cfg(test)]
mod serde_tests {
    use std::time::Duration;

    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::{TimeRange, TrainingParticipationReportCommand};

    #[test]
    fn serialize_enable_presence_logging() {
        let cmd = TrainingParticipationReportCommand::EnablePresenceLogging {
            initial_checkpoint_delay: Some(TimeRange::new_with_clamped_durations(
                Duration::from_secs(123),
                Duration::from_secs(456),
            )),
            checkpoint_interval: None,
        };
        let raw = serde_json::to_string_pretty(&cmd).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "enable_presence_logging",
          "initial_checkpoint_delay": {
            "after": 123,
            "within": 456
          }
        }
        "#);
    }

    #[test]
    fn deserialize_enable_presence_logging() {
        let json = json!({
            "action": "enable_presence_logging",
        });

        assert_eq!(
            serde_json::from_value::<TrainingParticipationReportCommand>(json).unwrap(),
            TrainingParticipationReportCommand::EnablePresenceLogging {
                initial_checkpoint_delay: None,
                checkpoint_interval: None
            }
        );
    }

    #[test]
    fn deserialize_enable_presence_logging_with_params() {
        let json = json!({
            "action": "enable_presence_logging",
            "initial_checkpoint_delay": {
                "after": 600,
                "within": 1200,
            },
            "checkpoint_interval": {
                "after": 6300,
                "within": 1800,
            }
        });

        assert_eq!(
            serde_json::from_value::<TrainingParticipationReportCommand>(json).unwrap(),
            TrainingParticipationReportCommand::EnablePresenceLogging {
                initial_checkpoint_delay: Some(TimeRange::new_with_clamped_durations(
                    Duration::from_mins(10),
                    Duration::from_mins(20)
                )),
                checkpoint_interval: Some(TimeRange::new_with_clamped_durations(
                    Duration::from_mins(105),
                    Duration::from_mins(30)
                )),
            }
        );
    }

    #[test]
    fn serialize_disable_presence_logging() {
        let cmd = TrainingParticipationReportCommand::DisablePresenceLogging;
        let raw = serde_json::to_string_pretty(&cmd).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "disable_presence_logging"
        }
        "#);
    }

    #[test]
    fn deserialize_disable_presence_logging() {
        let json = json!({
            "action": "disable_presence_logging",
        });

        assert_eq!(
            serde_json::from_value::<TrainingParticipationReportCommand>(json).unwrap(),
            TrainingParticipationReportCommand::DisablePresenceLogging
        );
    }

    #[test]
    fn serialize_confirm_presence() {
        let cmd = TrainingParticipationReportCommand::ConfirmPresence;
        let raw = serde_json::to_string_pretty(&cmd).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "confirm_presence"
        }
        "#);
    }

    #[test]
    fn deserialize_confirm_presence() {
        let json = json!({
            "action": "confirm_presence",
        });

        assert_eq!(
            serde_json::from_value::<TrainingParticipationReportCommand>(json).unwrap(),
            TrainingParticipationReportCommand::ConfirmPresence
        );
    }
}

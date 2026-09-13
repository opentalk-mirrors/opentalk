// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_types::module_settings::SignalingModuleSettings;
use opentalk_types_common::{
    modules::ModuleId, training_participation_report::TrainingParticipationReportParameterSet,
    utils::ExampleData,
};
use serde::{Deserialize, Serialize};

use crate::TRAINING_PARTICIPATION_REPORT_MODULE_ID;

/// Training participation report settings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainingParticipationReportSettings {
    /// If [`Some`], participation logging will start automatically with the given parameters
    pub autostart: Option<TrainingParticipationReportParameterSet>,
}

impl SignalingModuleSettings for TrainingParticipationReportSettings {
    const NAMESPACE: ModuleId = TRAINING_PARTICIPATION_REPORT_MODULE_ID;
}

impl ExampleData for TrainingParticipationReportSettings {
    fn example_data() -> Self {
        Self {
            autostart: Some(TrainingParticipationReportParameterSet::example_data()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use insta::assert_snapshot;
    use opentalk_types_common::training_participation_report::TimeRange;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_training_participation_report_settings() {
        let settings = TrainingParticipationReportSettings {
            autostart: Some(TrainingParticipationReportParameterSet {
                initial_checkpoint_delay: TimeRange::new_with_clamped_durations(
                    Duration::from_secs(100),
                    Duration::from_secs(200),
                ),
                checkpoint_interval: TimeRange::new_with_clamped_durations(
                    Duration::from_secs(300),
                    Duration::from_secs(400),
                ),
            }),
        };
        let raw = serde_json::to_string_pretty(&settings).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "autostart": {
            "initial_checkpoint_delay": {
              "after": 100,
              "within": 200
            },
            "checkpoint_interval": {
              "after": 300,
              "within": 400
            }
          }
        }
        "#);
    }

    #[test]
    fn deserialize_training_participation_report_settings() {
        let json = json!({
            "autostart": {
                "initial_checkpoint_delay": {
                    "after": 100,
                    "within": 200
                },
                "checkpoint_interval": {
                    "after": 300,
                    "within": 400
                }
            }
        });

        let produced: TrainingParticipationReportSettings = serde_json::from_value(json).unwrap();
        let expected = TrainingParticipationReportSettings {
            autostart: Some(TrainingParticipationReportParameterSet {
                initial_checkpoint_delay: TimeRange::new_with_clamped_durations(
                    Duration::from_secs(100),
                    Duration::from_secs(200),
                ),
                checkpoint_interval: TimeRange::new_with_clamped_durations(
                    Duration::from_secs(300),
                    Duration::from_secs(400),
                ),
            }),
        };

        assert_eq!(produced, expected);
    }
}

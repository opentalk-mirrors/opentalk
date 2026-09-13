// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use super::TimeRange;
use crate::utils::ExampleData;

/// The parameters for a training participant report checkpoint procedure.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "redis",
    derive(redis_args::ToRedisArgs, redis_args::FromRedisValue)
)]
#[cfg_attr(feature = "redis", to_redis_args(serde), from_redis_value(serde))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(TrainingParticipationReportParameterSet::example_data())),
)]
#[cfg_attr(
    feature = "typescript",
    derive(ts_rs::TS),
    ts(export_to = "training-participation-report/")
)]
pub struct TrainingParticipationReportParameterSet {
    /// The time range definition for the initial checkpoint delay.
    pub initial_checkpoint_delay: TimeRange,

    /// The time range definition for the subsequent checkpoints.
    pub checkpoint_interval: TimeRange,
}

impl ExampleData for TrainingParticipationReportParameterSet {
    fn example_data() -> Self {
        Self {
            initial_checkpoint_delay: TimeRange::new_with_clamped_durations(
                Duration::from_secs(100),
                Duration::from_secs(200),
            ),
            checkpoint_interval: TimeRange::new_with_clamped_durations(
                Duration::from_secs(300),
                Duration::from_secs(400),
            ),
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::{
        training_participation_report::TrainingParticipationReportParameterSet,
        utils::ExampleData as _,
    };

    #[test]
    fn deserialize_parameter_set_example() {
        let json = json!({
            "initial_checkpoint_delay": {
                "after": 100,
                "within": 200,
            },
            "checkpoint_interval": {
                "after": 300,
                "within": 400,
            },
        });

        assert_eq!(
            serde_json::from_value::<TrainingParticipationReportParameterSet>(json).unwrap(),
            TrainingParticipationReportParameterSet::example_data()
        );
    }

    #[test]
    fn serialize_time_range_example() {
        assert_eq!(
            json!(TrainingParticipationReportParameterSet::example_data()),
            json!({
                "initial_checkpoint_delay": {
                    "after": 100,
                    "within": 200,
                },
                "checkpoint_interval": {
                    "after": 300,
                    "within": 400,
                },
            })
        );
    }

    #[test]
    fn deserialize_invalid_initial_checkpoint_delay() {
        let json = json!({
            "initial_checkpoint_delay": "abc",
            "checkpoint_interval": {
                "after": 100,
                "within": 200,
            },
        });

        assert!(serde_json::from_value::<TrainingParticipationReportParameterSet>(json).is_err());
    }

    #[test]
    fn deserialize_invalid_checkpoint_interval() {
        let json = json!({
            "initial_checkpoint_delay": {
                "after": 100,
                "within": 200,
            },
            "checkpoint_interval": "abc",
        });

        assert!(serde_json::from_value::<TrainingParticipationReportParameterSet>(json).is_err());
    }
}

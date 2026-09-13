// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use crate::utils::ExampleData;

/// The minimum initial delay until the first checkpoint is reached. This prevents a DoS attack in
/// which a user could spam the creation of reports.
const MIN_INITIAL_DELAY: Duration = Duration::from_mins(1);
/// The maximum initial delay until the first checkpoint is reached.
const MAX_INITIAL_DELAY: Duration = Duration::from_hours(10);
/// The maximum time window within which a checkpoint can be created.
const MAX_TIME_WINDOW: Duration = Duration::from_hours(10);

/// Defines a time window within which a checkpoint can be created.
///
/// The checkpoint can be created after a minimum delay (`after`) has elapsed, and is created
/// randomly within a maximum duration (`within`) after the initial delay.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(TimeRange::example_data())),
)]
#[cfg_attr(
    feature = "typescript",
    derive(ts_rs::TS),
    ts(export_to = "training-participation-report/")
)]
pub struct TimeRange {
    /// The shortest duration that needs to elapse before the checkpoint can be created, represented in seconds.
    #[cfg_attr(feature = "serde", serde(with = "crate::utils::duration_seconds"))]
    #[cfg_attr(
        feature = "utoipa",
        schema(value_type = u32, minimum = 60, maximum = 36000)
    )]
    #[cfg_attr(feature = "typescript", ts(type = "number"))]
    after: Duration,

    /// The time window within which the checkpoint is created, represented in seconds.
    #[cfg_attr(feature = "serde", serde(with = "crate::utils::duration_seconds"))]
    #[cfg_attr(
        feature = "utoipa",
        schema(value_type = u32, minimum = 0, maximum = 36000)
    )]
    #[cfg_attr(feature = "typescript", ts(type = "number"))]
    within: Duration,
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for TimeRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct TimeRangeHelper {
            #[serde(with = "crate::utils::duration_seconds")]
            after: Duration,
            #[serde(with = "crate::utils::duration_seconds")]
            within: Duration,
        }

        let helper = TimeRangeHelper::deserialize(deserializer)?;

        if helper.after < MIN_INITIAL_DELAY {
            return Err(serde::de::Error::custom(format!(
                "`after` must be at least {MIN_INITIAL_DELAY:?}"
            )));
        }

        if helper.after > MAX_INITIAL_DELAY {
            return Err(serde::de::Error::custom(format!(
                "`after` must be at most {MAX_INITIAL_DELAY:?}",
            )));
        }

        if helper.within > MAX_TIME_WINDOW {
            return Err(serde::de::Error::custom(format!(
                "`within` must be at most {MAX_TIME_WINDOW:?} seconds"
            )));
        }

        Ok(TimeRange {
            after: helper.after,
            within: helper.within,
        })
    }
}

impl TimeRange {
    /// Creates a new [`TimeRange`]. Clamps the given durations to their respective minimum and
    /// maximum values.
    pub fn new_with_clamped_durations(after: Duration, within: Duration) -> Self {
        Self {
            after: after.clamp(MIN_INITIAL_DELAY, MAX_INITIAL_DELAY),
            within: within.min(MAX_TIME_WINDOW),
        }
    }

    /// The shortest duration that needs to elapse before the checkpoint can be created.
    pub fn after(&self) -> Duration {
        self.after
    }

    /// The time window within which the checkpoint is created.
    pub fn within(&self) -> Duration {
        self.within
    }
}

impl ExampleData for TimeRange {
    fn example_data() -> Self {
        Self {
            after: Duration::from_mins(20),
            within: Duration::from_mins(10),
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use std::time::Duration;

    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn time_range_clamping() {
        let time_range = TimeRange::new_with_clamped_durations(Duration::ZERO, Duration::ZERO);
        assert_eq!(time_range.after(), MIN_INITIAL_DELAY);
        assert_eq!(time_range.within(), Duration::ZERO);

        let time_range = TimeRange::new_with_clamped_durations(
            Duration::from_hours(1000),
            Duration::from_hours(1000),
        );
        assert_eq!(time_range.after(), MAX_INITIAL_DELAY);
        assert_eq!(time_range.within(), MAX_TIME_WINDOW);

        let time_range = TimeRange::new_with_clamped_durations(
            Duration::from_secs(120),
            Duration::from_hours(5),
        );
        assert_eq!(time_range.after(), Duration::from_secs(120));
        assert_eq!(time_range.within(), Duration::from_hours(5));
    }

    #[test]
    fn deserialize_time_range_zero() {
        let json = json!({
            "after": 60,
            "within": 0,
        });

        assert_eq!(
            serde_json::from_value::<TimeRange>(json).unwrap(),
            TimeRange {
                after: MIN_INITIAL_DELAY,
                within: Duration::ZERO,
            }
        );
    }

    #[test]
    fn serialize_time_range_zero() {
        assert_eq!(
            json!(TimeRange {
                after: Duration::from_mins(1),
                within: Duration::ZERO,
            }),
            json!({
                "after": 60,
                "within": 0,
            })
        );
    }

    #[test]
    fn deserialize_time_range_example() {
        let json = json!({
            "after": 1200,
            "within": 600,
        });

        assert_eq!(
            serde_json::from_value::<TimeRange>(json).unwrap(),
            TimeRange::example_data()
        );
    }

    #[test]
    fn serialize_time_range_example() {
        assert_eq!(
            json!(TimeRange::example_data()),
            json!({
                "after": 1200,
                "within": 600,
            })
        );
    }

    #[test]
    fn deserialize_invalid_below_minimum_after() {
        let json = json!({
            "after": 59,
            "within": 600,
        });

        assert!(serde_json::from_value::<TimeRange>(json).is_err());
    }

    #[test]
    fn deserialize_invalid_above_maximum_after() {
        let json = json!({
            "after": 36001,
            "within": 600,
        });

        assert!(serde_json::from_value::<TimeRange>(json).is_err());
    }

    #[test]
    fn deserialize_invalid_negative_after() {
        let json = json!({
            "after": -1200,
            "within": 600,
        });

        assert!(serde_json::from_value::<TimeRange>(json).is_err());
    }

    #[test]
    fn deserialize_invalid_negative_within() {
        let json = json!({
            "after": 1200,
            "within": -600,
        });

        assert!(serde_json::from_value::<TimeRange>(json).is_err());
    }

    #[test]
    fn deserialize_invalid_above_maximum_within() {
        let json = json!({
            "after": 1200,
            "within": 36001,
        });

        assert!(serde_json::from_value::<TimeRange>(json).is_err());
    }

    #[test]
    fn deserialize_invalid_missing_after() {
        let json = json!({
            "within": 600,
        });

        assert!(serde_json::from_value::<TimeRange>(json).is_err());
    }

    #[test]
    fn deserialize_invalid_missing_within() {
        let json = json!({
            "after": 1200,
        });

        assert!(serde_json::from_value::<TimeRange>(json).is_err());
    }
}

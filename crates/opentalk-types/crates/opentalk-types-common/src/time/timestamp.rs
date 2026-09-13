// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{ops::Add, time::SystemTime};

use chrono::{DateTime, TimeZone as _, Timelike as _, Utc};
use derive_more::{AsRef, Deref, Display, From, FromStr};

use crate::{time::DateTimeTz, utils::ExampleData};

/// A UTC DateTime wrapper that implements ToRedisArgs and FromRedisValue.
///
/// The values are stores as unix timestamps in redis.
#[derive(
    AsRef,
    Deref,
    Display,
    From,
    FromStr,
    Debug,
    Default,
    Copy,
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export_to = "common/"))]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    /// Create a timestamp with the date of the unix epoch start
    /// (1970-01-01 00:00:00 UTC)
    pub fn unix_epoch() -> Self {
        Self(DateTime::from(std::time::UNIX_EPOCH))
    }

    /// Create a timestamp with the current system time
    pub fn now() -> Timestamp {
        Timestamp(Utc::now())
    }

    /// Format as a string that can be used in a filename easily
    pub fn to_string_for_filename(&self) -> String {
        // UTC is the only supported timezone for now so we can hardcode
        // it because inserting timezone names is extra work due to
        // https://github.com/chronotope/chrono/issues/960
        self.0.format("%F_%H-%M-%S-UTC").to_string()
    }

    /// Round the timestamp to full seconds
    pub fn rounded_to_seconds(self) -> Timestamp {
        // This can only fail if the nanoseconds have an invalid value, 0 is
        // valid here
        Timestamp(self.0.with_nanosecond(0).expect("nanoseconds should be 0"))
    }
}

impl ExampleData for Timestamp {
    fn example_data() -> Self {
        Timestamp(Utc.with_ymd_and_hms(2024, 7, 20, 14, 16, 19).unwrap())
    }
}

impl From<SystemTime> for Timestamp {
    fn from(value: SystemTime) -> Self {
        Self(value.into())
    }
}

impl From<DateTimeTz> for Timestamp {
    fn from(value: DateTimeTz) -> Self {
        value.datetime.into()
    }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(value: Timestamp) -> Self {
        value.0
    }
}

impl Add<chrono::Duration> for Timestamp {
    type Output = Timestamp;

    fn add(self, rhs: chrono::Duration) -> Self::Output {
        Timestamp(self.0 + rhs)
    }
}

#[cfg(feature = "redis")]
impl redis::ToRedisArgs for Timestamp {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        self.0.timestamp().write_redis_args(out)
    }

    fn describe_numeric_behavior(&self) -> redis::NumericBehavior {
        redis::NumericBehavior::NumberIsInteger
    }
}

#[cfg(feature = "redis")]
impl redis::ToSingleRedisArg for Timestamp {}

#[cfg(feature = "redis")]
impl redis::FromRedisValue for Timestamp {
    fn from_redis_value(v: redis::Value) -> Result<Timestamp, redis::ParsingError> {
        use chrono::TimeZone as _;
        let timestamp = Utc
            .timestamp_opt(i64::from_redis_value(v)?, 0)
            .latest()
            .unwrap();
        Ok(Timestamp(timestamp))
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone as _, Utc};

    use super::Timestamp;

    #[test]
    fn to_string_for_filename() {
        let timestamp = Timestamp::unix_epoch();
        assert_eq!(
            "1970-01-01_00-00-00-UTC",
            timestamp.to_string_for_filename().as_str()
        );

        let timestamp = Timestamp(Utc.with_ymd_and_hms(2020, 5, 3, 14, 16, 19).unwrap());
        assert_eq!(
            "2020-05-03_14-16-19-UTC",
            timestamp.to_string_for_filename().as_str()
        );
    }
}

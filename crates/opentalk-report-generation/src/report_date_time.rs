// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, NaiveDateTime, SubsecRound, TimeZone};
use chrono_tz::Tz;
use serde::{Deserialize, Deserializer, Serialize};
use snafu::{ResultExt as _, Snafu, ensure};

/// Representation of a date-time for use in report generation
///
/// Use any of the provided methods to generate this type, and embed
/// it in the data used to generate a report.
///
/// For easier creation, there is the [`ToReportDateTime`] trait which is implemented
/// on [`DateTime`] and [`Option<DateTime>`].
///
/// This type wraps a [`NaiveDateTime`] which is rounded to seconds, removing
/// the subsecond part, because the method used to parse timestamps in the
/// report generation does not support that.
///
/// The idea is to represent all timestamps in a report with the same timezone.
/// Therefore when creating the [`ReportDateTime`] type, the timezone used in
/// the report must be passed in as a parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ReportDateTime(NaiveDateTime);

impl ReportDateTime {
    /// Build a [`ReportDateTime`] from a [`chrono::DateTime`], converting it to
    /// the timezone used in the report.
    pub fn from_date_time_for_tz<TZ: TimeZone>(dt: DateTime<TZ>, report_tz: &Tz) -> Self {
        Self(dt.round_subsecs(0).with_timezone(report_tz).naive_local())
    }
}

impl Display for ReportDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The error that is returned by [ModuleId::from_str] on failure.
#[derive(Debug, Snafu)]
pub enum ParseReportDateTimeError {
    /// NaiveDateTime parsing failed
    #[snafu(display("Could not parse NaiveDateTime"))]
    NaiveDateTime { source: chrono::format::ParseError },

    /// The ReportDateTime type must be rounded to seconds
    #[snafu(display("Must be rounded to seconds"))]
    NotRoundedToSeconds,
}

impl FromStr for ReportDateTime {
    type Err = ParseReportDateTimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dt: NaiveDateTime = s.parse().context(NaiveDateTimeSnafu)?;
        ensure!(dt.round_subsecs(0) == dt, NotRoundedToSecondsSnafu);
        Ok(Self(dt))
    }
}

impl<'de> Deserialize<'de> for ReportDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let dt: NaiveDateTime = Deserialize::deserialize(deserializer)?;
        if dt.round_subsecs(0) != dt {
            return Err(serde::de::Error::custom(
                "report date time must be rounded to full seconds",
            ));
        }
        Ok(Self(dt))
    }
}

/// A trait for easy conversion of datetime related types into the types that
/// should be used in the report.
pub trait ToReportDateTime {
    /// The output type of the conversion when applied to the input type.
    type Output;

    /// Convert to the [`Self::Output`] type based on the timezone that should
    /// be used in the report generation.
    fn to_report_date_time(&self, report_tz: &Tz) -> Self::Output;
}

impl<TZ: TimeZone> ToReportDateTime for DateTime<TZ> {
    type Output = ReportDateTime;

    fn to_report_date_time(&self, report_tz: &Tz) -> Self::Output {
        ReportDateTime::from_date_time_for_tz(self.clone(), report_tz)
    }
}

impl<TZ: TimeZone> ToReportDateTime for Option<DateTime<TZ>> {
    type Output = Option<ReportDateTime>;

    fn to_report_date_time(&self, report_tz: &Tz) -> Self::Output {
        self.clone().map(|dt| dt.to_report_date_time(report_tz))
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::ReportDateTime;
    use crate::ToReportDateTime as _;

    #[test]
    fn from_str_to_string() {
        let s = "2025-01-08T11:12:13";
        let parsed: ReportDateTime = s.parse().expect("value must be parsable as ReportDateTime");
        assert_eq!(parsed.to_string(), "2025-01-08 11:12:13");
    }

    #[test]
    fn parse_with_subseconds() {
        let s = "2025-01-08T11:12:13.1234";
        let _check_can_be_deserialized_as_naive: NaiveDateTime =
            s.parse().expect("value must be parsable as NaiveDateTime");

        assert!(s.parse::<ReportDateTime>().is_err());
    }

    #[test]
    fn serialize() {
        let s = "2025-01-08T11:12:13";
        let parsed: ReportDateTime = s.parse().expect("value must be parsable as ReportDateTime");
        assert_eq!(json!(s), json!(parsed));
    }

    #[test]
    fn deserialize() {
        let s = "2025-01-08T11:12:13";
        let expected: ReportDateTime = s.parse().expect("value must be parsable as ReportDateTime");

        let deserialized: ReportDateTime = serde_json::from_value(json!(s))
            .expect("value must be valid json parseable as ReportDateTime");

        assert_eq!(expected, deserialized);
    }

    #[test]
    fn deserialize_with_subseconds() {
        let s = "2025-01-08T11:12:13.1234";
        let _check_can_be_deserialized_as_naive: NaiveDateTime =
            s.parse().expect("value must be parsable as NaiveDateTime");

        assert!(serde_json::from_value::<ReportDateTime>(json!(s)).is_err());
    }

    #[test]
    fn to_report_date_time_from_utc() {
        let dt: DateTime<Utc> = "2025-01-08T11:12:13.1234Z"
            .parse()
            .expect("value must be parsable as DateTime");

        let expected: ReportDateTime = "2025-01-08T12:12:13"
            .parse()
            .expect("value must be parsable as ReportDateTime");

        assert_eq!(expected, dt.to_report_date_time(&chrono_tz::Europe::Berlin));
    }

    #[test]
    fn to_report_date_time_from_tz_with_same_offset() {
        let dt: DateTime<Utc> = "2025-01-08T11:12:13.1234+01:00"
            .parse()
            .expect("value must be parsable as DateTime");

        let expected: ReportDateTime = "2025-01-08T11:12:13"
            .parse()
            .expect("value must be parsable as ReportDateTime");

        assert_eq!(expected, dt.to_report_date_time(&chrono_tz::Europe::Berlin));
    }

    #[test]
    fn to_report_date_time_option_none() {
        let dt: Option<DateTime<Utc>> = None;

        assert_eq!(None, dt.to_report_date_time(&chrono_tz::Europe::Berlin));
    }

    #[test]
    fn to_report_date_time_option_some() {
        let dt: Option<DateTime<Utc>> = Some(
            "2025-01-08T11:12:13.1234Z"
                .parse()
                .expect("value must be parsable as DateTime"),
        );

        let expected: Option<ReportDateTime> = Some(
            "2025-01-08T12:12:13"
                .parse()
                .expect("value must be parsable as ReportDateTime"),
        );

        assert_eq!(expected, dt.to_report_date_time(&chrono_tz::Europe::Berlin));
    }
}

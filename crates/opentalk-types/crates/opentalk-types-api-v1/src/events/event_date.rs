// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{TimeZone, Utc};
use opentalk_types_common::{
    time::{DateTimeTz, RecurrencePattern},
    utils::ExampleData,
};

/// Representing event date related information.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(
    example = json!(
        EventDate::example_data()
    )
))]
pub struct EventDate {
    /// Should the event be all-day?
    ///
    /// If true, requires `starts_at.datetime` and `ends_at.datetime` to have a 00:00 time part.
    #[cfg_attr(feature = "serde", serde(default))]
    pub is_all_day: bool,
    /// Start time of the event.
    ///
    /// For recurring events these must contain the datetime of the first instance.
    pub starts_at: DateTimeTz,
    /// End time of the event.
    ///
    /// For recurring events these must contain the datetime of the first instance.
    pub ends_at: DateTimeTz,
    /// Recurrence pattern(s) for recurring events.
    ///
    /// Contains a list of recurrence rules which describe the occurrences of the event.
    ///
    /// # Allowed Values
    ///
    /// - `RRULE`
    /// - `RDATE`
    /// - `EXRULE`
    /// - `EXDATE`
    ///
    /// # Forbidden Values
    ///
    /// - `DTSTART`
    /// - `DTEND`
    ///
    /// This information is stored in the `starts_at` and `ends_at` fields.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "RecurrencePattern::is_empty")
    )]
    pub recurrence_pattern: RecurrencePattern,
}

impl ExampleData for EventDate {
    fn example_data() -> Self {
        Self {
            is_all_day: true,
            starts_at: DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2002, 4, 1, 10, 41, 35).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            },
            ends_at: DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2002, 4, 1, 11, 41, 35).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            },
            recurrence_pattern: RecurrencePattern::example_data(),
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize() {
        let expected = json!({
            "is_all_day": true,
            "starts_at": {
                "datetime": "2002-04-01T10:41:35Z",
                "timezone": "Europe/Berlin",
            },
            "ends_at": {
                "datetime": "2002-04-01T11:41:35Z",
                "timezone": "Europe/Berlin",
            },
            "recurrence_pattern": RecurrencePattern::example_data(),
        });

        let produced = json!(EventDate {
            is_all_day: true,
            starts_at: DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2002, 4, 1, 10, 41, 35).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            },
            ends_at: DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2002, 4, 1, 11, 41, 35).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            },
            recurrence_pattern: RecurrencePattern::example_data(),
        });

        assert_eq!(expected, produced);
    }

    #[test]
    fn deserialize() {
        let expected = EventDate {
            is_all_day: true,
            starts_at: DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2002, 4, 1, 10, 41, 35).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            },
            ends_at: DateTimeTz {
                datetime: Utc.with_ymd_and_hms(2002, 4, 1, 11, 41, 35).unwrap(),
                timezone: chrono_tz::Europe::Berlin.into(),
            },
            recurrence_pattern: RecurrencePattern::example_data(),
        };

        let produced = serde_json::from_value(json!({
            "is_all_day": true,
            "starts_at": {
                "datetime": "2002-04-01T10:41:35Z",
                "timezone": "Europe/Berlin",
            },
            "ends_at": {
                "datetime": "2002-04-01T11:41:35Z",
                "timezone": "Europe/Berlin",
            },
            "recurrence_pattern": RecurrencePattern::example_data(),
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{time::DateTimeTz, utils::ExampleData};

use crate::events::{EventDate, TimeDependentMarker, TimeIndependentMarker};

/// Contains date related parameters for the respective event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(untagged, deny_unknown_fields)
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(
    example = json!(
        EventDateKind::example_data()
    )
))]
pub enum EventDateKind {
    /// Event is not bound to specific times.
    #[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
    TimeIndependent {
        /// Should the created event be time independent?
        ///
        /// If true, all following fields must be null.
        /// If false, requires `is_all_day`, `starts_at`, `ends_at`.
        is_time_independent: TimeIndependentMarker,
    },
    /// Event is bound to specific times.
    #[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
    TimeDependent {
        /// Marker field to identify this variant during deserialization.
        is_time_independent: TimeDependentMarker,
        /// Represents all date related information of the respective event.
        #[cfg_attr(feature = "serde", serde(flatten))]
        date: EventDate,
    },
}

impl EventDateKind {
    /// Default time independent event kind.
    pub const TIME_INDEPENDENT: EventDateKind = EventDateKind::TimeIndependent {
        is_time_independent: TimeIndependentMarker,
    };

    /// Get the event date for a time-dependent event, [`None`] for a time-independent event.
    pub const fn event_date(&self) -> Option<&EventDate> {
        let EventDateKind::TimeDependent { date, .. } = self else {
            return None;
        };
        Some(date)
    }

    /// Get the `is_all_day` field for a time-dependent event, [`None`] for a time-independent event.
    pub fn is_all_day(&self) -> Option<bool> {
        self.event_date().map(|d| d.is_all_day)
    }

    /// Get the `starts_at` field for a time-dependent event, [`None`] for a time-independent event.
    pub fn starts_at(&self) -> Option<&DateTimeTz> {
        self.event_date().map(|d| &d.starts_at)
    }

    /// Get the `ends_at` field for a time-dependent event, [`None`] for a time-independent event.
    pub fn ends_at(&self) -> Option<&DateTimeTz> {
        self.event_date().map(|d| &d.ends_at)
    }
}

impl Default for EventDateKind {
    fn default() -> Self {
        Self::TIME_INDEPENDENT
    }
}

impl From<EventDate> for EventDateKind {
    fn from(date: EventDate) -> Self {
        Self::TimeDependent {
            is_time_independent: TimeDependentMarker,
            date,
        }
    }
}

impl ExampleData for EventDateKind {
    fn example_data() -> Self {
        Self::TimeDependent {
            is_time_independent: TimeDependentMarker,
            date: EventDate::example_data(),
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use chrono::{TimeZone, Utc};
    use opentalk_types_common::time::{DateTimeTz, RecurrencePattern};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::events::EventDateKind;

    #[test]
    fn serialize_time_independent() {
        let expected = json!({
            "is_time_independent": true,
        });

        let produced = json!(EventDateKind::TIME_INDEPENDENT);

        assert_eq!(expected, produced);
    }

    #[test]
    fn deserialize_time_independent() {
        let expected = EventDateKind::TIME_INDEPENDENT;

        let produced = serde_json::from_value(json!({
            "is_time_independent": true,
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_time_dependent() {
        let expected = json!({
            "is_time_independent": false,
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

        let produced = json!(EventDateKind::TimeDependent {
            is_time_independent: TimeDependentMarker,
            date: EventDate {
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
            },
        });

        assert_eq!(expected, produced);
    }

    #[test]
    fn deserialize_time_dependent() {
        let expected = EventDateKind::TimeDependent {
            is_time_independent: TimeDependentMarker,
            date: EventDate {
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
            },
        };

        let produced = serde_json::from_value(json!({
            "is_time_independent": false,
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

    #[test]
    #[should_panic]
    fn deserialize_time_independent_with_date() {
        let _ = serde_json::from_value::<EventDateKind>(json!({
            "is_time_independent": true,
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
    }

    #[test]
    #[should_panic]
    fn deserialize_time_dependent_without_date() {
        let _ = serde_json::from_value::<EventDateKind>(json!({
            "is_time_independent": false,
        }))
        .unwrap();
    }
}

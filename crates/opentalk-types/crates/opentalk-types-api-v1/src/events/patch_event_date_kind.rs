// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::utils::ExampleData;

use crate::events::{EventDate, PatchEventDate, TimeDependentMarker, TimeIndependentMarker};

/// Contains date related parameters for the respective event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(untagged, deny_unknown_fields)
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(
    example = json!(
        PatchEventDateKind::example_data()
    )
))]
pub enum PatchEventDateKind {
    /// Change an event that was previously bound to a specific date to be time
    /// independent.
    ///
    /// If this is set, remove all date related fields from exisisting event.
    #[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
    SetTimeIndependent {
        /// Is the event time independent.
        is_time_independent: TimeIndependentMarker,
    },
    /// Change an event that was previouusly not bound to a specific date to be
    /// time dependent.
    #[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
    SetTimeDependent {
        /// Marker field to identify this variant as time dependent during
        /// deserialization.
        is_time_independent: TimeDependentMarker,
        /// Represents all date related information of the respective event.
        #[cfg_attr(feature = "serde", serde(flatten))]
        date: EventDate,
    },
    /// Patch an event that is already time dependent.
    #[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
    PatchTimeDependent {
        /// Optional marker field to identify this variant as time dependent
        /// during deserialization.
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        // Field is non-required already, utoipa adds a `nullable: true` entry
        // by default which creates a false positive in the spectral linter when
        // combined with example data.
        #[cfg_attr(feature = "utoipa", schema(nullable = false))]
        is_time_independent: Option<TimeDependentMarker>,
        /// Represents all optional date related information of the respective
        /// event.
        #[cfg_attr(feature = "serde", serde(flatten))]
        date: PatchEventDate,
    },
}

impl PatchEventDateKind {
    /// Default time independent event kind.
    pub const SET_TIME_INDEPENDENT: PatchEventDateKind = PatchEventDateKind::SetTimeIndependent {
        is_time_independent: TimeIndependentMarker,
    };

    /// Returns `true` if no date related fields are set.
    pub const fn is_empty(&self) -> bool {
        match self {
            PatchEventDateKind::SetTimeIndependent { .. } => false,
            PatchEventDateKind::SetTimeDependent { .. } => false,
            PatchEventDateKind::PatchTimeDependent {
                is_time_independent,
                date,
            } => is_time_independent.is_none() && date.is_empty(),
        }
    }
}

impl Default for PatchEventDateKind {
    fn default() -> Self {
        Self::SET_TIME_INDEPENDENT
    }
}

impl From<PatchEventDate> for PatchEventDateKind {
    fn from(date: PatchEventDate) -> Self {
        Self::PatchTimeDependent {
            is_time_independent: Some(TimeDependentMarker),
            date,
        }
    }
}

impl ExampleData for PatchEventDateKind {
    fn example_data() -> Self {
        Self::PatchTimeDependent {
            is_time_independent: Some(TimeDependentMarker),
            date: PatchEventDate::example_data(),
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
    use crate::events::{EventDateKind, TimeDependentMarker};

    #[test]
    fn serialize_set_time_independent() {
        let expected = json!({
            "is_time_independent": true,
        });

        let produced = json!(EventDateKind::TIME_INDEPENDENT);

        assert_eq!(expected, produced);
    }

    #[test]
    fn deserialize_set_time_independent() {
        let expected = EventDateKind::TIME_INDEPENDENT;

        let produced = serde_json::from_value(json!({
            "is_time_independent": true,
        }))
        .unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_set_time_dependent() {
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

        let produced = json!(PatchEventDateKind::SetTimeDependent {
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
    fn deserialize_set_time_dependent() {
        let expected = PatchEventDateKind::SetTimeDependent {
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
    fn serialize_patch_time_dependent() {
        let expected = json!({
            "is_all_day": true,
            "ends_at": {
                "datetime": "2002-04-01T11:41:35Z",
                "timezone": "Europe/Berlin",
            },
            "recurrence_pattern": RecurrencePattern::example_data(),
        });

        let produced = json!(PatchEventDateKind::PatchTimeDependent {
            is_time_independent: None,
            date: PatchEventDate {
                is_all_day: Some(true),
                starts_at: None,
                ends_at: Some(DateTimeTz {
                    datetime: Utc.with_ymd_and_hms(2002, 4, 1, 11, 41, 35).unwrap(),
                    timezone: chrono_tz::Europe::Berlin.into(),
                }),
                recurrence_pattern: RecurrencePattern::example_data(),
            },
        });

        assert_eq!(expected, produced);
    }

    #[test]
    fn deserialize_patch_time_dependent() {
        let expected = PatchEventDateKind::PatchTimeDependent {
            is_time_independent: None,
            date: PatchEventDate {
                is_all_day: Some(true),
                starts_at: None,
                ends_at: Some(DateTimeTz {
                    datetime: Utc.with_ymd_and_hms(2002, 4, 1, 11, 41, 35).unwrap(),
                    timezone: chrono_tz::Europe::Berlin.into(),
                }),
                recurrence_pattern: RecurrencePattern::example_data(),
            },
        };

        let produced = serde_json::from_value(json!({
            "is_all_day": true,
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

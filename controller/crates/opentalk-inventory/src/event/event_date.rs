// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::time::{TimeZone, Timestamp};

use crate::event::EventRecurrence;

/// Contains information about the date of a new event.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EventDate {
    /// A flag indicating whether this is an all-day event.
    pub is_all_day: bool,
    /// Start datetime of the event.
    pub starts_at: Timestamp,
    /// Timezone of the start-datetime of the event.
    pub starts_at_tz: TimeZone,
    /// End datetime of the event.
    ///
    /// For recurring events contains the timestamp of the last occurrence.
    pub ends_at: Timestamp,
    /// Timezone of the ends_at datetime.
    pub ends_at_tz: TimeZone,
    /// Only for recurring events, since ends_at contains the information
    /// about the last occurrence of the recurring series this duration
    /// value.
    ///
    /// MUST be used to calculate the event instances length.
    pub duration_secs: i32,
    /// The recurrence pattern for recurring events.
    pub recurrence: Option<EventRecurrence>,
}

impl EventDate {
    /// Provides the ends_at value of the first occurrence of the event:
    /// - if the event is not recurring: returns `ends_at` and `ends_at_tz` of the event
    /// - otherwise: returns `ends_at` and `ends_at_tz` of first occurence
    pub fn ends_at_of_first_occurrence(&self) -> (Timestamp, TimeZone) {
        if self.recurrence.is_none() {
            return (self.ends_at, self.ends_at_tz);
        };

        (
            self.starts_at + chrono::Duration::seconds(self.duration_secs as i64),
            self.ends_at_tz,
        )
    }
}

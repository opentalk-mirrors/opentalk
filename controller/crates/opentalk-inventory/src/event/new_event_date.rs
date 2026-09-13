// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::DateTime;
use chrono_tz::Tz;
use opentalk_types_common::time::TimeZone;

use crate::event::NewEventRecurrence;

/// Contains information about the date of a new event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewEventDate {
    /// A flag indicating whether this is an all-day event.
    pub is_all_day: bool,
    /// Start datetime of the event.
    pub starts_at: DateTime<Tz>,
    /// Timezone of the start-datetime of the event.
    pub starts_at_tz: TimeZone,
    /// End datetime of the event.
    ///
    /// For recurring events contains the timestamp of the last occurrence.
    pub ends_at: DateTime<Tz>,
    /// Timezone of the ends_at datetime.
    pub ends_at_tz: TimeZone,
    /// The recurrence pattern for recurring events.
    pub recurrence: Option<NewEventRecurrence>,
    /// Only for recurring events, since ends_at contains the information
    /// about the last occurrence of the recurring series this duration
    /// value.
    ///
    /// MUST be used to calculate the event instances length.
    pub duration_secs: i32,
}

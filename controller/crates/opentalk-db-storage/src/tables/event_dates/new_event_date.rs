// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::DateTime;
use chrono_tz::Tz;
use diesel::prelude::*;
use opentalk_types_common::{events::EventId, time::TimeZone};

use crate::schema::event_dates;

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = event_dates)]
pub struct NewEventDate {
    pub event_id: EventId,
    /// Start datetime of the event.
    pub starts_at: DateTime<Tz>,
    /// Timezone of the `starts_at` datetime.
    pub starts_at_tz: TimeZone,
    /// End datetime of the event.
    ///
    /// For recurring events contains the timestamp of the last occurrence.
    pub ends_at: DateTime<Tz>,
    /// Timezone of the `ends_at` datetime.
    pub ends_at_tz: TimeZone,
    /// Denotes whether an event is all day, meaning it starts at 00:00 and ends at 00:00 the
    /// following day.
    pub is_all_day: bool,
    /// Only for recurring events, since ends_at contains the information about the last occurrence
    /// of the recurring series this duration value.
    ///
    /// MUST be used to calculate the event instances length.
    pub duration_secs: i32,
}

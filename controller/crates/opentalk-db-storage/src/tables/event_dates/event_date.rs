// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use opentalk_types_common::{events::EventId, time::TimeZone};
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

use crate::{schema::event_dates, tables::events::Event};

#[derive(
    Associations,
    Clone,
    Debug,
    Deserialize,
    Eq,
    FromRedisValue,
    Identifiable,
    PartialEq,
    Queryable,
    Serialize,
    ToRedisArgs,
)]
#[diesel(table_name = event_dates)]
#[diesel(belongs_to(Event, foreign_key = event_id))]
#[diesel(primary_key(event_id))]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
pub struct EventDate {
    /// ID of the event the date belongs to.
    pub event_id: EventId,
    /// Start datetime of the event.
    pub starts_at: DateTime<Utc>,
    /// Timezone of the `starts_at` datetime.
    pub starts_at_tz: TimeZone,
    /// End datetime of the event.
    ///
    /// For recurring events contains the timestamp of the last occurrence.
    pub ends_at: DateTime<Utc>,
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

impl EventDate {
    /// Returns the starts at of this [`EventDate`].
    pub fn starts_at(&self) -> DateTime<Utc> {
        self.starts_at
    }
}

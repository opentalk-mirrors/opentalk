// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::prelude::*;
use opentalk_types_common::events::EventId;
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

use crate::{schema::event_recurrences, tables::event_dates::EventDate};

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
#[diesel(table_name = event_recurrences)]
#[diesel(belongs_to(EventDate, foreign_key = event_id))]
#[diesel(primary_key(event_id))]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
pub struct EventRecurrence {
    /// ID of the event the date belongs to.
    pub event_id: EventId,
    /// Recurrence pattern of the event.
    pub recurrence_pattern: String,
}

impl EventRecurrence {
    /// Returns the recurrence pattern of this [`EventDate`].
    pub fn recurrence_pattern(&self) -> &str {
        self.recurrence_pattern.as_str()
    }
}

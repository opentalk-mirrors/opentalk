// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::prelude::*;
use opentalk_types_common::events::EventId;

use crate::schema::event_recurrences;

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = event_recurrences)]
pub struct NewEventRecurrence {
    pub event_id: EventId,
    /// Recurrence pattern of the event.
    pub recurrence_pattern: String,
}

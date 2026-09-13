// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::prelude::*;

use crate::schema::event_recurrences;

#[derive(Debug, Clone, AsChangeset, Insertable)]
#[diesel(table_name = event_recurrences)]
pub struct UpdateEventRecurrence {
    /// Recurrence pattern of the event.
    pub recurrence_pattern: String,
}

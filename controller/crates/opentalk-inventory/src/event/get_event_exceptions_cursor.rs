// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{events::EventId, time::Timestamp};

use crate::EventException;

/// A cursor for getting event exceptions starting at a certain position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetEventExceptionsCursor {
    from_id: EventId,
    from_created_at: Timestamp,
    from_starts_at: Option<Timestamp>,
    from_exception_date: Timestamp,
}

impl GetEventExceptionsCursor {
    /// Create a new [`GetEventExceptionsCursor`].
    pub fn new(
        from_id: EventId,
        from_created_at: Timestamp,
        from_starts_at: Option<Timestamp>,
        from_exception_date: Timestamp,
    ) -> Self {
        Self {
            from_id,
            from_created_at,
            from_starts_at,
            from_exception_date,
        }
    }

    /// Create a cursor starting at a specific event exception.
    pub fn from_last_event(exception: &EventException) -> Self {
        Self {
            from_id: exception.event_id,
            from_created_at: exception.created_at,
            from_starts_at: exception.starts_at,
            from_exception_date: exception.exception_date,
        }
    }
}

impl From<GetEventExceptionsCursor> for (EventId, Timestamp, Option<Timestamp>, Timestamp) {
    fn from(
        GetEventExceptionsCursor {
            from_id,
            from_created_at,
            from_starts_at,
            from_exception_date,
        }: GetEventExceptionsCursor,
    ) -> Self {
        (
            from_id,
            from_created_at,
            from_starts_at,
            from_exception_date,
        )
    }
}

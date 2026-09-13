// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{events::EventId, time::Timestamp};

use super::Event;

/// A cursor for getting events starting at a certain position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetEventsCursor {
    from_id: EventId,
    from_created_at: Timestamp,
    from_starts_at: Option<Timestamp>,
}

impl GetEventsCursor {
    /// Create a new [`GetEventsCursor`].
    pub fn new(
        from_id: EventId,
        from_created_at: Timestamp,
        from_starts_at: Option<Timestamp>,
    ) -> Self {
        Self {
            from_id,
            from_created_at,
            from_starts_at,
        }
    }

    /// Create a cursor starting at a specific event.
    pub fn from_last_event(event: &Event) -> Self {
        Self {
            from_id: event.id,
            from_created_at: event.created_at,
            from_starts_at: event.starts_at(),
        }
    }
}

impl From<GetEventsCursor> for (EventId, Timestamp, Option<Timestamp>) {
    fn from(
        GetEventsCursor {
            from_id,
            from_created_at,
            from_starts_at,
        }: GetEventsCursor,
    ) -> Self {
        (from_id, from_created_at, from_starts_at)
    }
}

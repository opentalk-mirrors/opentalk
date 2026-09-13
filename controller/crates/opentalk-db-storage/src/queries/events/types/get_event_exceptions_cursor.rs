// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use opentalk_inventory as inventory;
use opentalk_types_common::events::EventId;

use crate::tables::event_exceptions::EventException;

pub struct GetEventExceptionsCursor {
    pub from_id: EventId,
    pub from_created_at: DateTime<Utc>,
    pub from_starts_at: Option<DateTime<Utc>>,
    pub from_exception_date: DateTime<Utc>,
}

impl From<GetEventExceptionsCursor> for inventory::GetEventExceptionsCursor {
    fn from(
        GetEventExceptionsCursor {
            from_id,
            from_created_at,
            from_starts_at,
            from_exception_date,
        }: GetEventExceptionsCursor,
    ) -> Self {
        Self::new(
            from_id,
            from_created_at.into(),
            from_starts_at.map(Into::into),
            from_exception_date.into(),
        )
    }
}

impl From<inventory::GetEventExceptionsCursor> for GetEventExceptionsCursor {
    fn from(value: inventory::GetEventExceptionsCursor) -> Self {
        let (from_id, from_created_at, from_starts_at, from_exception_date) = value.into();
        Self {
            from_id,
            from_created_at: from_created_at.into(),
            from_starts_at: from_starts_at.map(Into::into),
            from_exception_date: from_exception_date.into(),
        }
    }
}

impl GetEventExceptionsCursor {
    pub fn from_last_event_in_query(exception: &EventException) -> Self {
        Self {
            from_id: exception.event_id,
            from_created_at: exception.created_at,
            from_starts_at: exception.starts_at,
            from_exception_date: exception.exception_date,
        }
    }
}

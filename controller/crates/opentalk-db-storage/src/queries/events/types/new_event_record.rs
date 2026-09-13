// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_inventory as inventory;
use opentalk_types_common::events::EventId;

use crate::tables::{
    event_dates::NewEventDate, event_recurrences::NewEventRecurrence, events::NewEvent,
};

#[derive(Debug, Clone)]
pub struct NewEventRecord {
    /// New event to be inserted into the db.
    event: NewEvent,
    /// Raw event date without event id.
    raw_date: Option<inventory::NewEventDate>,
}

impl NewEventRecord {
    pub fn build_date(self, event_id: EventId) -> Option<NewEventDateRecord> {
        let inventory::NewEventDate {
            is_all_day,
            starts_at,
            starts_at_tz,
            ends_at,
            ends_at_tz,
            recurrence,
            duration_secs,
        } = self.raw_date?;

        Some(NewEventDateRecord {
            date: NewEventDate {
                event_id,
                starts_at,
                starts_at_tz,
                ends_at,
                ends_at_tz,
                is_all_day,
                duration_secs,
            },
            raw_recurrence: recurrence,
        })
    }

    /// Returns a reference to the event of this [`NewEventRecord`].
    pub fn event(&self) -> &NewEvent {
        &self.event
    }
}

pub struct NewEventDateRecord {
    /// New event to be inserted into the db.
    date: NewEventDate,
    /// Raw event date without event id.
    raw_recurrence: Option<inventory::NewEventRecurrence>,
}

impl NewEventDateRecord {
    pub fn build_recurrence(self) -> Option<NewEventRecurrence> {
        let inventory::NewEventRecurrence { recurrence_pattern } = self.raw_recurrence?;

        Some(NewEventRecurrence {
            event_id: self.date.event_id,
            recurrence_pattern,
        })
    }

    /// Returns a reference to the date of this [`NewEventDateRecord`].
    pub fn date(&self) -> &NewEventDate {
        &self.date
    }
}

impl From<inventory::NewEvent> for NewEventRecord {
    fn from(
        inventory::NewEvent {
            title,
            description,
            room,
            created_by,
            updated_by,
            is_adhoc,
            tenant_id,
            show_meeting_details,
            date,
        }: inventory::NewEvent,
    ) -> Self {
        Self {
            event: NewEvent {
                title,
                description,
                room,
                created_by,
                updated_by,
                is_adhoc,
                tenant_id,
                show_meeting_details,
            },
            raw_date: date,
        }
    }
}

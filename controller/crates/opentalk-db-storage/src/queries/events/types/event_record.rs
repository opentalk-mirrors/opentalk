// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_inventory as inventory;
use opentalk_types_common::events::EventId;

use crate::tables::{event_dates::EventDate, event_recurrences::EventRecurrence, events::Event};

/// Composite type that combines both [`Event`], an optional [`EventDate`] and an optional
/// [`EventRecurrence`]. This is used for the translation into [`opentalk_inventory::Event`].
#[derive(Debug, Clone, PartialEq, Queryable)]
pub struct EventRecord {
    /// Core event related fields.
    pub event: Event,
    /// Event date related fields.
    pub date: Option<EventDate>,
    /// Event recurrence related fields.
    pub recurrence: Option<EventRecurrence>,
}

impl EventRecord {
    /// Creates a new [`EventRecord`].
    pub fn new(event: Event, date: Option<EventDate>, recurrence: Option<EventRecurrence>) -> Self {
        Self {
            event,
            date,
            recurrence,
        }
    }

    /// Returns a reference to the event of this [`EventRecord`].
    pub fn event(&self) -> &Event {
        &self.event
    }

    /// Returns the id of this [`EventRecord`].
    pub fn id(&self) -> EventId {
        self.event().id
    }

    /// Returns the date of this [`EventRecord`].
    pub fn date(&self) -> Option<&EventDate> {
        self.date.as_ref()
    }

    /// Returns the recurrence of this [`EventRecord`].
    pub fn recurrence(&self) -> Option<&EventRecurrence> {
        self.recurrence.as_ref()
    }
}

impl From<inventory::Event> for EventRecord {
    fn from(
        inventory::Event {
            id,
            id_serial,
            title,
            description,
            room,
            created_by,
            created_at,
            updated_by,
            updated_at,
            is_adhoc,
            tenant_id,
            revision,
            show_meeting_details,
            date,
        }: inventory::Event,
    ) -> Self {
        let event = Event {
            id,
            id_serial: id_serial.into(),
            title,
            description,
            room,
            created_by,
            created_at: created_at.into(),
            updated_by,
            updated_at: updated_at.into(),
            is_adhoc,
            tenant_id,
            revision,
            show_meeting_details,
        };

        let Some(inventory::EventDate {
            is_all_day,
            starts_at,
            starts_at_tz,
            ends_at,
            ends_at_tz,
            recurrence,
            duration_secs,
        }) = date
        else {
            return Self::new(event, None, None);
        };

        let date = Some(EventDate {
            event_id: id,
            starts_at: starts_at.into(),
            starts_at_tz,
            ends_at: ends_at.into(),
            ends_at_tz,
            is_all_day,
            duration_secs,
        });

        let Some(inventory::EventRecurrence { recurrence_pattern }) = recurrence else {
            return Self::new(event, date, None);
        };

        let recurrence = Some(EventRecurrence {
            event_id: id,
            recurrence_pattern,
        });

        Self::new(event, date, recurrence)
    }
}

impl From<EventRecord> for inventory::Event {
    fn from(
        EventRecord {
            event:
                Event {
                    id,
                    id_serial,
                    title,
                    description,
                    room,
                    created_by,
                    created_at,
                    updated_by,
                    updated_at,
                    is_adhoc,
                    tenant_id,
                    revision,
                    show_meeting_details,
                },
            date,
            recurrence,
        }: EventRecord,
    ) -> inventory::Event {
        let recurrence = recurrence.map(
            |EventRecurrence {
                 event_id: _,
                 recurrence_pattern,
             }| { inventory::EventRecurrence { recurrence_pattern } },
        );

        let date = date.map(
            |EventDate {
                 event_id: _,
                 starts_at,
                 starts_at_tz,
                 ends_at,
                 ends_at_tz,
                 is_all_day,
                 duration_secs,
             }| {
                inventory::EventDate {
                    is_all_day,
                    starts_at: starts_at.into(),
                    starts_at_tz,
                    ends_at: ends_at.into(),
                    ends_at_tz,
                    recurrence,
                    duration_secs,
                }
            },
        );

        inventory::Event {
            id,
            id_serial: id_serial.into(),
            title,
            description,
            room,
            created_by,
            created_at: created_at.into(),
            updated_by,
            updated_at: updated_at.into(),
            is_adhoc,
            tenant_id,
            revision,
            show_meeting_details,
            date,
        }
    }
}

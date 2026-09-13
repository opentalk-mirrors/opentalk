// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_inventory as inventory;

use crate::tables::{
    event_dates::UpdateEventDate, event_recurrences::UpdateEventRecurrence, events::UpdateEvent,
};

/// Composite type that combines [`UpdateEvent`], an optional [`UpdateEventDate`] and an optional
/// [`UpdateEventRecurrence`].
#[derive(Debug, Clone)]
pub struct UpdateEventRecord {
    /// Core event related fields.
    event: UpdateEvent,
    /// Event date related fields.
    date: Option<UpdateEventDate>,
    /// Event recurrence related fields.
    recurrence: Option<UpdateEventRecurrence>,
}

impl UpdateEventRecord {
    /// Creates a new [`UpdateEventRecord`].
    pub fn new(
        event: UpdateEvent,
        date: Option<UpdateEventDate>,
        recurrence: Option<UpdateEventRecurrence>,
    ) -> Self {
        Self {
            event,
            date,
            recurrence,
        }
    }

    /// Returns the date changeset of this [`UpdateEventRecord`].
    pub fn date(&self) -> Option<&UpdateEventDate> {
        self.date.as_ref()
    }

    /// Returns the recurrence changeset of this [`UpdateEventRecord`].
    pub fn recurrence(&self) -> Option<&UpdateEventRecurrence> {
        self.recurrence.as_ref()
    }

    /// Returns a reference to the event of this [`UpdateEventRecord`].
    pub fn event(&self) -> &UpdateEvent {
        &self.event
    }
}

impl From<inventory::UpdateEvent> for UpdateEventRecord {
    fn from(
        inventory::UpdateEvent {
            title,
            description,
            updated_by,
            updated_at,
            is_adhoc,
            show_meeting_details,
            date,
        }: inventory::UpdateEvent,
    ) -> UpdateEventRecord {
        let event = UpdateEvent {
            title,
            description,
            updated_by,
            updated_at: updated_at.into(),
            is_adhoc,
            show_meeting_details,
        };

        let Some(inventory::UpdateEventDate {
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

        let date = Some(UpdateEventDate {
            starts_at,
            starts_at_tz,
            ends_at,
            ends_at_tz,
            is_all_day,
            duration_secs,
        });

        let Some(inventory::UpdateEventRecurrence { recurrence_pattern }) = recurrence else {
            return Self::new(event, date, None);
        };

        let recurrence = Some(UpdateEventRecurrence { recurrence_pattern });

        Self::new(event, date, recurrence)
    }
}

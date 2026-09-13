// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::DateTime;
use chrono_tz::Tz;
use opentalk_types_common::{
    events::{EventDescription, EventTitle},
    rooms::RoomId,
    tenants::TenantId,
    time::TimeZone,
    users::UserId,
};

use crate::{NewEventRecurrence, event::new_event_date::NewEventDate};

/// The representation of a new event that is intended to be stored in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewEvent {
    /// The title of the event.
    pub title: EventTitle,

    /// The description of the event.
    pub description: EventDescription,

    /// The id of the room associated with the event.
    pub room: RoomId,

    /// The id of the user who created the event.
    pub created_by: UserId,

    /// The id of the user last updated the event.
    pub updated_by: UserId,

    /// A flag indicating whether this is an ad-hoc event.
    pub is_adhoc: bool,

    /// The id of the tenant to which the event belongs.
    pub tenant_id: TenantId,

    /// A flag indicating whether the details should be shown in the meeting.
    pub show_meeting_details: bool,

    /// Contains all date related information about the event.
    pub date: Option<NewEventDate>,
}

impl NewEvent {
    /// Returns `true` if this [`NewEvent`] has no date field associated.
    pub fn is_time_independent(&self) -> bool {
        self.date.is_none()
    }

    /// Returns the `date` of this [`NewEvent`].
    pub fn date(&self) -> Option<&NewEventDate> {
        self.date.as_ref()
    }

    /// Returns the `is_all_day` of this [`NewEvent`].
    pub fn is_all_day(&self) -> Option<bool> {
        self.date().map(|date| date.is_all_day)
    }

    /// Returns the `starts_at` of this [`NewEvent`].
    pub fn starts_at(&self) -> Option<DateTime<Tz>> {
        self.date().map(|date| date.starts_at)
    }

    /// Returns the `ends_at` of this [`NewEvent`].
    pub fn ends_at(&self) -> Option<DateTime<Tz>> {
        self.date().map(|date| date.ends_at)
    }

    /// Returns the `starts_at_tz` of this [`NewEvent`].
    pub fn starts_at_tz(&self) -> Option<TimeZone> {
        self.date().map(|date| date.starts_at_tz)
    }

    /// Returns the `ends_at_tz` of this [`NewEvent`].
    pub fn ends_at_tz(&self) -> Option<TimeZone> {
        self.date().map(|date| date.ends_at_tz)
    }

    /// Returns the `duration_secs` of this [`NewEvent`].
    pub fn duration_secs(&self) -> Option<i32> {
        self.date().map(|date| date.duration_secs)
    }

    /// Returns the `recurrence` of this [`NewEvent`].
    pub fn recurrence(&self) -> Option<&NewEventRecurrence> {
        self.date().and_then(|date| date.recurrence.as_ref())
    }

    /// Returns the `recurrence_pattern` of this [`NewEvent`].
    pub fn recurrence_pattern(&self) -> Option<&str> {
        self.recurrence()
            .map(|recurrence| recurrence.recurrence_pattern.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone as _;
    use opentalk_types_common::utils::ExampleData;

    use super::*;

    #[test]
    fn is_time_independent() {
        let time_independent_event = NewEvent {
            title: EventTitle::example_data(),
            description: EventDescription::example_data(),
            room: RoomId::example_data(),
            created_by: UserId::example_data(),
            updated_by: UserId::example_data(),
            is_adhoc: false,
            tenant_id: TenantId::nil(),
            show_meeting_details: false,
            date: None,
        };

        assert!(time_independent_event.is_time_independent());

        let time_dependent_event = NewEvent {
            title: EventTitle::example_data(),
            description: EventDescription::example_data(),
            room: RoomId::example_data(),
            created_by: UserId::example_data(),
            updated_by: UserId::example_data(),
            is_adhoc: false,
            tenant_id: TenantId::nil(),
            show_meeting_details: false,
            date: Some(NewEventDate {
                is_all_day: false,
                starts_at: Tz::default()
                    .with_ymd_and_hms(2024, 7, 20, 14, 16, 19)
                    .unwrap(),
                starts_at_tz: TimeZone::default(),
                ends_at: Tz::default()
                    .with_ymd_and_hms(2024, 7, 20, 14, 16, 19)
                    .unwrap(),
                ends_at_tz: TimeZone::default(),
                duration_secs: 1800,
                recurrence: None,
            }),
        };

        assert!(!time_dependent_event.is_time_independent());
    }
}

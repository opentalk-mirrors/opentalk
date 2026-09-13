// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    events::{EventDescription, EventId, EventTitle},
    rooms::RoomId,
    tenants::TenantId,
    time::{TimeZone, Timestamp},
    users::UserId,
};

use crate::{EventDate, EventRecurrence};

/// The representation of an event in the inventory.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Event {
    /// The id of the event.
    pub id: EventId,

    /// The serial id of the event.
    pub id_serial: i64,

    /// The title of the event.
    pub title: EventTitle,

    /// The description of the event.
    pub description: EventDescription,

    /// The id of the room associated with the event.
    pub room: RoomId,

    /// The id of the user who created the event.
    pub created_by: UserId,

    /// The creation timestamp.
    pub created_at: Timestamp,

    /// The id of the user last updated the event.
    pub updated_by: UserId,

    /// The updated timestamp.
    pub updated_at: Timestamp,

    /// A flag indicating whether this is an ad-hoc event.
    pub is_adhoc: bool,

    /// The id of the tenant to which the event belongs.
    pub tenant_id: TenantId,

    /// The revision number of the event, incremented with each change.
    pub revision: i32,

    /// A flag indicating whether the details should be shown in the meeting.
    pub show_meeting_details: bool,

    /// Contains all date related information about the event.
    pub date: Option<EventDate>,
}

impl Event {
    /// Provides the ends_at value of the first occurrence of the event:
    /// - if the event has no date: returns none
    /// - if the event is not recurring: returns some `ends_at` and `ends_at_tz` of the event
    /// - otherwise: returns some `ends_at` and `ends_at_tz` of first occurence
    pub fn ends_at_of_first_occurrence(&self) -> Option<(Timestamp, TimeZone)> {
        self.date().map(|date| date.ends_at_of_first_occurrence())
    }

    /// Returns the date of this [`Event`].
    pub fn date(&self) -> Option<&EventDate> {
        self.date.as_ref()
    }

    /// Returns `false` if the [`Event`] has a date.
    pub fn is_time_independent(&self) -> bool {
        self.date().is_none()
    }

    /// Returns the starts_at of this [`Event`].
    pub fn starts_at(&self) -> Option<Timestamp> {
        self.date().map(|date| date.starts_at)
    }

    /// Returns the ends_at of this [`Event`].
    pub fn ends_at(&self) -> Option<Timestamp> {
        self.date().map(|date| date.ends_at)
    }

    // Note: TimeZone is Copy, so we don't need the ref.
    /// Returns the starts_at_tz of this [`Event`].
    pub fn starts_at_tz(&self) -> Option<TimeZone> {
        self.date().map(|date| date.starts_at_tz)
    }

    /// Returns the ends_at_tz of this [`Event`].
    pub fn ends_at_tz(&self) -> Option<TimeZone> {
        self.date().map(|date| date.ends_at_tz)
    }

    /// Returns the is_all_day of this [`Event`].
    pub fn is_all_day(&self) -> Option<bool> {
        self.date().map(|date| date.is_all_day)
    }

    /// Returns the duration_secs of this [`Event`].
    pub fn duration_secs(&self) -> Option<i32> {
        self.date().map(|date| date.duration_secs)
    }

    /// Returns the recurrence of this [`Event`].
    pub fn recurrence(&self) -> Option<&EventRecurrence> {
        self.date().and_then(|date| date.recurrence.as_ref())
    }

    /// Returns the recurrence_pattern of this [`Event`].
    pub fn recurrence_pattern(&self) -> Option<&str> {
        self.recurrence()
            .map(|recurrence| recurrence.recurrence_pattern.as_ref())
    }

    /// Returns `true` if the [`Event`] has a recurrence.
    pub fn is_recurring(&self) -> bool {
        self.recurrence().is_some()
    }
}

#[cfg(test)]
mod tests {
    use opentalk_types_common::utils::ExampleData;

    use super::*;

    #[test]
    fn is_time_independent() {
        let time_independent_event = Event {
            id: EventId::nil(),
            id_serial: 0,
            created_at: Timestamp::example_data(),
            updated_at: Timestamp::example_data(),
            title: EventTitle::example_data(),
            description: EventDescription::example_data(),
            room: RoomId::example_data(),
            created_by: UserId::example_data(),
            updated_by: UserId::example_data(),
            is_adhoc: false,
            tenant_id: TenantId::nil(),
            show_meeting_details: false,
            date: None,
            revision: 1,
        };

        assert!(time_independent_event.is_time_independent());

        let time_dependent_event = Event {
            id: EventId::nil(),
            id_serial: 0,
            created_at: Timestamp::example_data(),
            updated_at: Timestamp::example_data(),
            title: EventTitle::example_data(),
            description: EventDescription::example_data(),
            room: RoomId::example_data(),
            created_by: UserId::example_data(),
            updated_by: UserId::example_data(),
            is_adhoc: false,
            tenant_id: TenantId::nil(),
            show_meeting_details: false,
            revision: 1,
            date: Some(EventDate {
                is_all_day: false,
                starts_at: Timestamp::example_data(),
                starts_at_tz: TimeZone::default(),
                ends_at: Timestamp::example_data(),
                ends_at_tz: TimeZone::default(),
                recurrence: None,
                duration_secs: 1800,
            }),
        };

        assert!(!time_dependent_event.is_time_independent());
    }

    #[test]
    fn is_recurring() {
        let single_event = Event {
            id: EventId::nil(),
            id_serial: 0,
            created_at: Timestamp::example_data(),
            updated_at: Timestamp::example_data(),
            title: EventTitle::example_data(),
            description: EventDescription::example_data(),
            room: RoomId::example_data(),
            created_by: UserId::example_data(),
            updated_by: UserId::example_data(),
            is_adhoc: false,
            tenant_id: TenantId::nil(),
            show_meeting_details: false,
            revision: 1,
            date: Some(EventDate {
                is_all_day: false,
                starts_at: Timestamp::example_data(),
                starts_at_tz: TimeZone::default(),
                ends_at: Timestamp::example_data(),
                ends_at_tz: TimeZone::default(),
                duration_secs: 1800,
                recurrence: None,
            }),
        };

        assert!(!single_event.is_recurring());

        let recurring_event = Event {
            id: EventId::nil(),
            id_serial: 0,
            created_at: Timestamp::example_data(),
            updated_at: Timestamp::example_data(),
            title: EventTitle::example_data(),
            description: EventDescription::example_data(),
            room: RoomId::example_data(),
            created_by: UserId::example_data(),
            updated_by: UserId::example_data(),
            is_adhoc: false,
            tenant_id: TenantId::nil(),
            show_meeting_details: false,
            revision: 1,
            date: Some(EventDate {
                is_all_day: false,
                starts_at: Timestamp::example_data(),
                starts_at_tz: TimeZone::default(),
                ends_at: Timestamp::example_data(),
                ends_at_tz: TimeZone::default(),
                duration_secs: 10000,
                recurrence: Some(EventRecurrence {
                    recurrence_pattern: "FREQ=DAILY;INTERVAL=1".to_string(),
                }),
            }),
        };

        assert!(recurring_event.is_recurring());
    }
}

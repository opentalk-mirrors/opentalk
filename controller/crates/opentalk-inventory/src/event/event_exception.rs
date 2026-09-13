// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    events::{EventDescription, EventId, EventTitle},
    time::{TimeZone, Timestamp},
    users::UserId,
};

use super::{EventExceptionId, EventExceptionKind};

/// The representation of an event exception in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventException {
    /// The id of the event exception.
    pub id: EventExceptionId,

    /// The id of the event.
    pub event_id: EventId,

    /// The timestamp for which the exception applies.
    pub exception_date: Timestamp,

    /// The timezone of the exception date.
    pub exception_date_tz: TimeZone,

    /// The id of the use rwho created the event exception.
    pub created_by: UserId,

    /// The creation timestamp.
    pub created_at: Timestamp,

    /// The kind of event exception.
    pub kind: EventExceptionKind,

    /// An optional title changed by the event exception.
    pub title: Option<EventTitle>,

    /// An optional description changed by the event exception.
    pub description: Option<EventDescription>,

    /// An optional is_all_day flag changed by the exception.
    pub is_all_day: Option<bool>,

    /// An optional starts_at timestamp changed by the event exception.
    pub starts_at: Option<Timestamp>,

    /// An optional starts_at timezone changed by the event exception.
    pub starts_at_tz: Option<TimeZone>,

    /// An optional ends_at timestamp changed by the event exception.
    pub ends_at: Option<Timestamp>,

    /// An optional ends_at timezone changed by the event exception.
    pub ends_at_tz: Option<TimeZone>,
}

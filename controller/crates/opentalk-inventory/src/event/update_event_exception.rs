// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::DateTime;
use chrono_tz::Tz;
use opentalk_types_common::{
    events::{EventDescription, EventTitle},
    time::TimeZone,
};

use super::EventExceptionKind;

/// Representation of an update to an [`super::EventException`] in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateEventException {
    /// The kind of event exception.
    pub kind: Option<EventExceptionKind>,

    /// An optional title changed by the event exception.
    pub title: Option<Option<EventTitle>>,

    /// An optional description changed by the event exception.
    pub description: Option<Option<EventDescription>>,

    /// An optional is_all_day flag changed by the exception.
    pub is_all_day: Option<Option<bool>>,

    /// An optional starts_at timestamp changed by the event exception.
    pub starts_at: Option<Option<DateTime<Tz>>>,

    /// An optional starts_at timezone changed by the event exception.
    pub starts_at_tz: Option<Option<TimeZone>>,

    /// An optional ends_at timestamp changed by the event exception.
    pub ends_at: Option<Option<DateTime<Tz>>>,

    /// An optional ends_at timezone changed by the event exception.
    pub ends_at_tz: Option<Option<TimeZone>>,
}

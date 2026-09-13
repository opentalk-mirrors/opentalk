// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use diesel::Insertable;
use opentalk_inventory as inventory;
use opentalk_types_common::{
    events::{EventDescription, EventId, EventTitle},
    time::TimeZone,
    users::UserId,
};

use crate::{schema::event_exceptions, tables::event_exceptions::EventExceptionKind};

#[derive(Debug, Insertable)]
#[diesel(table_name = event_exceptions)]
pub struct NewEventException {
    pub event_id: EventId,
    pub exception_date: DateTime<Utc>,
    pub exception_date_tz: TimeZone,
    pub created_by: UserId,
    pub kind: EventExceptionKind,
    pub title: Option<EventTitle>,
    pub description: Option<EventDescription>,
    pub is_all_day: Option<bool>,
    pub starts_at: Option<DateTime<Tz>>,
    pub starts_at_tz: Option<TimeZone>,
    pub ends_at: Option<DateTime<Tz>>,
    pub ends_at_tz: Option<TimeZone>,
}

impl From<NewEventException> for inventory::NewEventException {
    fn from(
        NewEventException {
            event_id,
            exception_date,
            exception_date_tz,
            created_by,
            kind,
            title,
            description,
            is_all_day,
            starts_at,
            starts_at_tz,
            ends_at,
            ends_at_tz,
        }: NewEventException,
    ) -> Self {
        Self {
            event_id,
            exception_date: exception_date.into(),
            exception_date_tz,
            created_by,
            kind: kind.into(),
            title,
            description,
            is_all_day,
            starts_at,
            starts_at_tz,
            ends_at,
            ends_at_tz,
        }
    }
}

impl From<inventory::NewEventException> for NewEventException {
    fn from(
        inventory::NewEventException {
            event_id,
            exception_date,
            exception_date_tz,
            created_by,
            kind,
            title,
            description,
            is_all_day,
            starts_at,
            starts_at_tz,
            ends_at,
            ends_at_tz,
        }: inventory::NewEventException,
    ) -> Self {
        Self {
            event_id,
            exception_date: exception_date.into(),
            exception_date_tz,
            created_by,
            kind: kind.into(),
            title,
            description,
            is_all_day,
            starts_at,
            starts_at_tz,
            ends_at,
            ends_at_tz,
        }
    }
}

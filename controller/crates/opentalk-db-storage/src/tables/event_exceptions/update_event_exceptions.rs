// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::DateTime;
use chrono_tz::Tz;
use opentalk_inventory as inventory;
use opentalk_types_common::{
    events::{EventDescription, EventTitle},
    time::TimeZone,
};

use crate::{schema::event_exceptions, tables::event_exceptions::EventExceptionKind};

#[derive(AsChangeset, Debug)]
#[diesel(table_name = event_exceptions)]
pub struct UpdateEventException {
    pub kind: Option<EventExceptionKind>,
    pub title: Option<Option<EventTitle>>,
    pub description: Option<Option<EventDescription>>,
    pub is_all_day: Option<Option<bool>>,
    pub starts_at: Option<Option<DateTime<Tz>>>,
    pub starts_at_tz: Option<Option<TimeZone>>,
    pub ends_at: Option<Option<DateTime<Tz>>>,
    pub ends_at_tz: Option<Option<TimeZone>>,
}

impl From<inventory::UpdateEventException> for UpdateEventException {
    fn from(
        inventory::UpdateEventException {
            kind,
            title,
            description,
            is_all_day,
            starts_at,
            starts_at_tz,
            ends_at,
            ends_at_tz,
        }: inventory::UpdateEventException,
    ) -> Self {
        Self {
            kind: kind.map(Into::into),
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

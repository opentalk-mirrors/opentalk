// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use opentalk_types_common::{
    events::{EventDescription, EventTitle},
    users::UserId,
};

use crate::schema::events;

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = events)]
pub struct UpdateEvent {
    pub title: Option<EventTitle>,
    pub description: Option<EventDescription>,
    pub updated_by: UserId,
    pub updated_at: DateTime<Utc>,
    pub is_adhoc: Option<bool>,
    pub show_meeting_details: Option<bool>,
}

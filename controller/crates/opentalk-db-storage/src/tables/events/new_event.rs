// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::Insertable;
use opentalk_types_common::{
    events::{EventDescription, EventTitle},
    rooms::RoomId,
    tenants::TenantId,
    users::UserId,
};

use crate::schema::events;

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = events)]
pub struct NewEvent {
    pub title: EventTitle,
    pub description: EventDescription,
    pub room: RoomId,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub is_adhoc: bool,
    pub tenant_id: TenantId,
    pub show_meeting_details: bool,
}

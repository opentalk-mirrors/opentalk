// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use opentalk_inventory as inventory;
use opentalk_types_common::{
    events::{EventDescription, EventId, EventTitle},
    rooms::RoomId,
    tenants::TenantId,
    users::UserId,
};
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

use crate::{
    schema::events,
    tables::{events::EventSerialId, rooms::Room, users::User},
};

#[derive(
    Associations,
    Clone,
    Debug,
    Deserialize,
    Eq,
    FromRedisValue,
    Identifiable,
    PartialEq,
    Queryable,
    Serialize,
    ToRedisArgs,
)]
#[diesel(table_name = events)]
#[diesel(belongs_to(User, foreign_key = created_by))]
#[diesel(belongs_to(Room, foreign_key = room))]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
pub struct Event {
    pub id: EventId,
    pub id_serial: EventSerialId,
    pub title: EventTitle,
    pub description: EventDescription,
    pub room: RoomId,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_by: UserId,
    pub updated_at: DateTime<Utc>,
    pub is_adhoc: bool,
    pub tenant_id: TenantId,
    pub revision: i32,
    pub show_meeting_details: bool,
}

impl From<inventory::Event> for Event {
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
            date: _,
        }: inventory::Event,
    ) -> Self {
        Self {
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
        }
    }
}

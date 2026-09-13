// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use opentalk_inventory as inventory;
use opentalk_types_common::{
    events::{
        EventId,
        invites::{EventInviteStatus, InviteRole},
    },
    users::UserId,
};

use crate::{
    schema::event_invites,
    tables::{event_invites::EventInviteId, events::Event, users::User},
};

#[derive(Associations, Debug, Identifiable, Queryable)]
#[diesel(table_name = event_invites)]
#[diesel(belongs_to(Event, foreign_key = event_id))]
#[diesel(belongs_to(User, foreign_key = invitee))]
pub struct EventInvite {
    pub id: EventInviteId,
    pub event_id: EventId,
    pub invitee: UserId,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub status: EventInviteStatus,
    pub role: InviteRole,
}

impl From<EventInvite> for inventory::EventInvite {
    fn from(
        EventInvite {
            id,
            event_id,
            invitee,
            created_by,
            created_at,
            status,
            role,
        }: EventInvite,
    ) -> Self {
        Self {
            id: id.into(),
            event_id,
            invitee,
            created_by,
            created_at: created_at.into(),
            status,
            role,
        }
    }
}

impl From<inventory::EventInvite> for EventInvite {
    fn from(
        inventory::EventInvite {
            id,
            event_id,
            invitee,
            created_by,
            created_at,
            status,
            role,
        }: inventory::EventInvite,
    ) -> Self {
        Self {
            id: id.into(),
            event_id,
            invitee,
            created_by,
            created_at: created_at.into(),
            status,
            role,
        }
    }
}

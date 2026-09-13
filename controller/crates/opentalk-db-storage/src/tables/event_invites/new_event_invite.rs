// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use diesel::Insertable;
use opentalk_inventory as inventory;
use opentalk_types_common::{
    events::{EventId, invites::InviteRole},
    users::UserId,
};

use crate::schema::event_invites;

#[derive(Insertable)]
#[diesel(table_name = event_invites)]
pub struct NewEventInvite {
    pub event_id: EventId,
    pub invitee: UserId,
    pub role: InviteRole,
    pub created_by: UserId,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<inventory::NewEventInvite> for NewEventInvite {
    fn from(
        inventory::NewEventInvite {
            event_id,
            invitee,
            role,
            created_by,
            created_at,
        }: inventory::NewEventInvite,
    ) -> Self {
        Self {
            event_id,
            invitee,
            role,
            created_by,
            created_at: created_at.map(Into::into),
        }
    }
}

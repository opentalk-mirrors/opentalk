// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use opentalk_inventory as inventory;
use opentalk_types_common::{
    events::{EventId, invites::EmailInviteRole},
    users::UserId,
};

use crate::{schema::event_email_invites, tables::events::Event};

#[derive(Debug, Associations, Identifiable, Queryable)]
#[diesel(table_name = event_email_invites)]
#[diesel(primary_key(event_id, email))]
#[diesel(belongs_to(Event))]
pub struct EventEmailInvite {
    pub event_id: EventId,
    pub email: String,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub role: EmailInviteRole,
}

impl From<EventEmailInvite> for inventory::EventEmailInvite {
    fn from(
        EventEmailInvite {
            event_id,
            email,
            created_by,
            created_at,
            role,
        }: EventEmailInvite,
    ) -> Self {
        Self {
            event_id,
            email,
            created_by,
            created_at: created_at.into(),
            role,
        }
    }
}

impl From<inventory::EventEmailInvite> for EventEmailInvite {
    fn from(
        inventory::EventEmailInvite {
            event_id,
            email,
            created_by,
            created_at,
            role,
        }: inventory::EventEmailInvite,
    ) -> Self {
        Self {
            event_id,
            email,
            created_by,
            created_at: created_at.into(),
            role,
        }
    }
}

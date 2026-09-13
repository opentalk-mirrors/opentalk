// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::Insertable;
use opentalk_inventory as inventory;
use opentalk_types_common::{
    events::{EventId, invites::EmailInviteRole},
    users::UserId,
};

use crate::schema::event_email_invites;

#[derive(Insertable)]
#[diesel(table_name = event_email_invites)]
pub struct NewEventEmailInvite {
    pub event_id: EventId,
    pub email: String,
    pub role: EmailInviteRole,
    pub created_by: UserId,
}

impl From<inventory::NewEventEmailInvite> for NewEventEmailInvite {
    fn from(
        inventory::NewEventEmailInvite {
            event_id,
            email,
            role,
            created_by,
        }: inventory::NewEventEmailInvite,
    ) -> Self {
        Self {
            event_id,
            email,
            role,
            created_by,
        }
    }
}

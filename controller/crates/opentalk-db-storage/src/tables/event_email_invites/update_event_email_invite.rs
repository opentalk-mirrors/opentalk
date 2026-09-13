// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_inventory as inventory;
use opentalk_types_common::events::invites::EmailInviteRole;

use crate::schema::event_email_invites;

#[derive(AsChangeset)]
#[diesel(table_name = event_email_invites)]
pub struct UpdateEventEmailInvite {
    pub role: Option<EmailInviteRole>,
}

impl From<inventory::UpdateEventEmailInvite> for UpdateEventEmailInvite {
    fn from(inventory::UpdateEventEmailInvite { role }: inventory::UpdateEventEmailInvite) -> Self {
        Self { role }
    }
}

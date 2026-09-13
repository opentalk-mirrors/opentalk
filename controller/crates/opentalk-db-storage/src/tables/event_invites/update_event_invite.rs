// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_inventory as inventory;
use opentalk_types_common::events::invites::{EventInviteStatus, InviteRole};

use crate::schema::event_invites;

#[derive(AsChangeset)]
#[diesel(table_name = event_invites)]
pub struct UpdateEventInvite {
    pub status: Option<EventInviteStatus>,
    pub role: Option<InviteRole>,
}

impl From<inventory::UpdateEventInvite> for UpdateEventInvite {
    fn from(inventory::UpdateEventInvite { status, role }: inventory::UpdateEventInvite) -> Self {
        Self { status, role }
    }
}

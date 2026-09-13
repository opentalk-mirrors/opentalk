// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    events::{EventId, invites::EmailInviteRole},
    users::UserId,
};

/// The representation of a new event email invite that is intended to be stored in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewEventEmailInvite {
    /// The id of the event.
    pub event_id: EventId,

    /// The E-Mail address for which the invite is valid.
    pub email: String,

    /// The role of the invited person.
    pub role: EmailInviteRole,

    /// The id of the user who created the invite.
    pub created_by: UserId,
}

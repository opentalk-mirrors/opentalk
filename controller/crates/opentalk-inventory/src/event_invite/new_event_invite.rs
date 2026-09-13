// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    events::{EventId, invites::InviteRole},
    time::Timestamp,
    users::UserId,
};

/// The representation of a new event that is intended to be stored in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewEventInvite {
    /// The id of the event.
    pub event_id: EventId,

    /// The user id of the account to be invited.
    pub invitee: UserId,

    /// The role of the invitee.
    pub role: InviteRole,

    /// The id of the user who created the invite.
    pub created_by: UserId,

    /// The creation timestamp.
    pub created_at: Option<Timestamp>,
}

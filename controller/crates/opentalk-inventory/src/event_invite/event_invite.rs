// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    events::{
        EventId,
        invites::{EventInviteStatus, InviteRole},
    },
    time::Timestamp,
    users::UserId,
};

use super::EventInviteId;

/// The representation of an event invite in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventInvite {
    /// The id of the event invite.
    pub id: EventInviteId,

    /// The id of the event.
    pub event_id: EventId,

    /// The id of the invitee.
    pub invitee: UserId,

    /// The id of the user who created the event invite.
    pub created_by: UserId,

    /// The creation timestamp.
    pub created_at: Timestamp,

    /// The status of the event invite.
    pub status: EventInviteStatus,

    /// The role of the invited user.
    pub role: InviteRole,
}

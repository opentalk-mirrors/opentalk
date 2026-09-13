// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    events::{EventId, invites::EmailInviteRole},
    time::Timestamp,
    users::UserId,
};

/// The representation of an event email invite in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventEmailInvite {
    /// The id of the event to which the invite belongs.
    pub event_id: EventId,

    /// The E-Mail address for which the invite is valid.
    pub email: String,

    /// The id of the user who created the invite.
    pub created_by: UserId,

    /// The creation timestamp.
    pub created_at: Timestamp,

    /// The role of the invited person.
    pub role: EmailInviteRole,
}

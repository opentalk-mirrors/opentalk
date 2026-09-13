// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    rooms::{RoomId, invite_codes::InviteCode},
    time::Timestamp,
    users::UserId,
};

/// The representation of a room invite in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomInvite {
    /// The invite code.
    pub invite_code: InviteCode,

    /// The serial id of the user.
    pub id_serial: i64,

    /// The id of the user who created the invite.
    pub created_by: UserId,

    /// The creation timestamp.
    pub created_at: Timestamp,

    /// The id of the user who updated the invite.
    pub updated_by: UserId,

    /// The updated timestamp.
    pub updated_at: Timestamp,

    /// The id of the room to which the invite belongs.
    pub room: RoomId,

    /// A flag indicating whether the invite is active.
    pub active: bool,

    /// An optional expiry timestmamp.
    pub expiration: Option<Timestamp>,
}

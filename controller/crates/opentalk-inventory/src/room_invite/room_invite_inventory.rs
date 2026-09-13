// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    rooms::{RoomId, invite_codes::InviteCode},
    time::Timestamp,
};

use super::RoomInvite;
use crate::Result;

/// A trait for retrieving and storing room invite entities.
#[async_trait::async_trait]
pub trait RoomInviteInventory {
    /// Get a room invite by the invite code.
    async fn get_room_invite(&mut self, invite_code: InviteCode) -> Result<RoomInvite>;

    /// Get the invite code
    async fn get_room_invites_with_room_inactive_or_expired_before(
        &mut self,
        expired_before: Timestamp,
    ) -> Result<Vec<(InviteCode, RoomId)>>;
}

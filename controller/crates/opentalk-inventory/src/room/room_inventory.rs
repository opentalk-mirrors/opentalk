// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    pagination::{ItemCount, Page, PageSize},
    rooms::{RoomId, RoomIdOrAlias},
    users::UserId,
};

use super::{NewRoom, Room, UpdateRoom};
use crate::{Result, User};

/// A trait for retrieving and storing room entities.
#[async_trait::async_trait]
pub trait RoomInventory {
    /// Create a new room.
    async fn create_room(&mut self, new_room: NewRoom) -> Result<Room>;

    /// Get a room by its id.
    async fn get_room(&mut self, room: &RoomIdOrAlias) -> Result<Room>;

    /// Check if a room exists.
    async fn exists_room(&mut self, room: &RoomIdOrAlias) -> Result<bool>;

    /// Get a room and its creator by its id or alias.
    async fn get_room_with_creator(&mut self, room: &RoomIdOrAlias) -> Result<(Room, User)>;

    /// Update a room.
    async fn update_room(&mut self, room: &RoomIdOrAlias, update: UpdateRoom) -> Result<Room>;

    /// Delete a room.
    async fn delete_room(&mut self, room_id: RoomId) -> Result<()>;

    /// Get all rooms that don't have an event associated.
    async fn get_all_orphaned_room_ids(&mut self) -> Result<Vec<RoomId>>;

    /// Get all rooms accessible to a specific user, paginated and with the creator user.
    async fn get_rooms_accessible_to_user_with_creator_paginated(
        &mut self,
        user: UserId,
        limit: PageSize,
        page: Page,
    ) -> Result<(Vec<(Room, User)>, ItemCount)>;
}

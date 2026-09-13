// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    rooms::{RoomId, RoomIdOrAlias},
    streaming::{RoomStreamingTarget, StreamingTarget, StreamingTargetId},
};

use crate::{Result, RoomStreamingTargetRecord, UpdateRoomStreamingTarget};

/// A trait for retrieving and storing room streaming target entities.
#[async_trait::async_trait]
pub trait RoomStreamingTargetInventory {
    /// Get all room streaming targets for a room.
    async fn get_room_streaming_targets(
        &mut self,
        room_id: RoomId,
    ) -> Result<Vec<RoomStreamingTarget>>;

    /// Get room streaming target record.
    async fn get_room_streaming_target_record(
        &mut self,
        room_id: RoomId,
        streaming_target_id: StreamingTargetId,
    ) -> Result<RoomStreamingTargetRecord>;

    /// Create a room streaming target.
    async fn create_room_streaming_target(
        &mut self,
        room_id: RoomId,
        streaming_target: StreamingTarget,
    ) -> Result<RoomStreamingTarget>;

    /// Update a room streaming target.
    async fn update_room_streaming_target(
        &mut self,
        room: &RoomIdOrAlias,
        streaming_target_id: StreamingTargetId,
        streaming_target: UpdateRoomStreamingTarget,
    ) -> Result<RoomStreamingTargetRecord>;

    /// Update a room streaming target.
    async fn delete_room_streaming_target(
        &mut self,
        room: &RoomIdOrAlias,
        streaming_target_id: StreamingTargetId,
    ) -> Result<()>;

    /// Replace the streaming targets for a room.
    async fn replace_room_streaming_targets(
        &mut self,
        room_id: RoomId,
        streaming_targets: Vec<StreamingTarget>,
    ) -> Result<Vec<RoomStreamingTarget>>;
}

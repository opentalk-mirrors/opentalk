// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage as db;
use opentalk_inventory::{
    RoomStreamingTargetInventory, RoomStreamingTargetRecord, UpdateRoomStreamingTarget,
};
use opentalk_types_common::{
    rooms::{RoomId, RoomIdOrAlias},
    streaming::{RoomStreamingTarget, StreamingTarget, StreamingTargetId},
};
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result, error::DatabaseSnafu};

#[async_trait::async_trait]
impl RoomStreamingTargetInventory for DatabaseConnection {
    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_room_streaming_targets(
        &mut self,
        room_id: RoomId,
    ) -> Result<Vec<RoomStreamingTarget>> {
        Ok(
            db::queries::streaming_targets::get_room_streaming_targets(&mut self.inner, room_id)
                .await
                .context(DatabaseSnafu)?
                .into_iter()
                .map(TryInto::try_into)
                .collect::<std::result::Result<Vec<_>, _>>()
                .context(DatabaseSnafu)?,
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_room_streaming_target_record(
        &mut self,
        room_id: RoomId,
        streaming_target_id: StreamingTargetId,
    ) -> Result<RoomStreamingTargetRecord> {
        Ok(db::queries::streaming_targets::get_room_streaming_target(
            &mut self.inner,
            streaming_target_id,
            room_id,
        )
        .await
        .context(DatabaseSnafu)?
        .into())
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn create_room_streaming_target(
        &mut self,
        room_id: RoomId,
        streaming_target: StreamingTarget,
    ) -> Result<RoomStreamingTarget> {
        Ok(
            db::queries::streaming_targets::create_room_streaming_target(
                &mut self.inner,
                room_id,
                streaming_target,
            )
            .await
            .context(DatabaseSnafu)?,
        )
    }

    async fn update_room_streaming_target(
        &mut self,
        room: &RoomIdOrAlias,
        streaming_target_id: StreamingTargetId,
        streaming_target: UpdateRoomStreamingTarget,
    ) -> Result<RoomStreamingTargetRecord> {
        Ok(
            db::queries::streaming_targets::update_room_streaming_target(
                &mut self.inner,
                streaming_target.into(),
                room,
                streaming_target_id,
            )
            .await
            .context(DatabaseSnafu)?
            .into(),
        )
    }

    async fn delete_room_streaming_target(
        &mut self,
        room: &RoomIdOrAlias,
        streaming_target_id: StreamingTargetId,
    ) -> Result<()> {
        Ok(
            db::queries::streaming_targets::delete_room_streaming_target(
                &mut self.inner,
                room,
                streaming_target_id,
            )
            .await
            .context(DatabaseSnafu)?,
        )
    }

    async fn replace_room_streaming_targets(
        &mut self,
        room_id: RoomId,
        streaming_targets: Vec<StreamingTarget>,
    ) -> Result<Vec<RoomStreamingTarget>> {
        Ok(
            db::queries::streaming_targets::replace_room_streaming_targets(
                &mut self.inner,
                room_id,
                streaming_targets,
            )
            .await
            .context(DatabaseSnafu)?,
        )
    }
}

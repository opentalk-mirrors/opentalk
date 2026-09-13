// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains streaming targets database queries

use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use opentalk_database::{DatabaseError, DbConnection, Result};
use opentalk_types_common::{
    self as types,
    rooms::{RoomId, RoomIdOrAlias},
    streaming::{StreamingTarget, StreamingTargetId},
};

use crate::{
    queries::room_filter::FilterByRoom as _,
    schema::{room_streaming_targets, rooms},
    tables::room_streaming_targets::{
        NewRoomStreamingTarget, RoomStreamingTarget, UpdateRoomStreamingTarget,
    },
};

pub async fn create_room_streaming_target(
    conn: &mut DbConnection,
    room_id: RoomId,
    streaming_target: StreamingTarget,
) -> Result<types::streaming::RoomStreamingTarget> {
    let streaming_target_record = insert(
        conn,
        NewRoomStreamingTarget::from_streaming_target_kind(
            streaming_target.kind.clone(),
            room_id,
            streaming_target.name.clone(),
        ),
    )
    .await?;

    Ok(types::streaming::RoomStreamingTarget {
        id: streaming_target_record.id,
        streaming_target,
    })
}

pub async fn replace_room_streaming_targets(
    conn: &mut DbConnection,
    room_id: RoomId,
    streaming_targets: Vec<StreamingTarget>,
) -> Result<Vec<types::streaming::RoomStreamingTarget>> {
    conn.transaction(async |conn| {
        // Delete existing records by room_id
        delete_by_room_id(conn, room_id).await?;

        let new_records: Vec<NewRoomStreamingTarget> = streaming_targets
            .into_iter()
            .map(|streaming_target| {
                NewRoomStreamingTarget::from_streaming_target_kind(
                    streaming_target.kind,
                    room_id,
                    streaming_target.name,
                )
            })
            .collect();

        // Insert new records and fetch the resulting records
        let inserted_records: Vec<RoomStreamingTarget> =
            diesel::insert_into(room_streaming_targets::table)
                .values(&new_records)
                .get_results(conn)
                .await?;

        Ok(inserted_records)
    })
    .await
    .and_then(|inserted_records| {
        // Transform inserted_records into RoomStreamingTarget
        inserted_records
            .into_iter()
            .map(|record| record.try_into())
            .collect::<Result<Vec<_>, DatabaseError>>()
    })
}

/// Retrieve a single streaming target
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_room_streaming_target(
    conn: &mut DbConnection,
    streaming_target_id: StreamingTargetId,
    room_id: RoomId,
) -> Result<RoomStreamingTarget> {
    room_streaming_targets::table
        .filter(room_streaming_targets::id.eq(streaming_target_id))
        .filter(room_streaming_targets::room_id.eq(room_id))
        .first(conn)
        .await
        .map_err(DatabaseError::from)
}

/// Retrieve all streaming targets
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_room_streaming_targets(
    conn: &mut DbConnection,
    room_id: RoomId,
) -> Result<Vec<RoomStreamingTarget>> {
    room_streaming_targets::table
        .filter(room_streaming_targets::room_id.eq(room_id))
        .load(conn)
        .await
        .map_err(DatabaseError::from)
}

/// Delete a streaming target using the given room & streaming target id
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn delete_room_streaming_target(
    conn: &mut DbConnection,
    room: &RoomIdOrAlias,
    streaming_target_id: StreamingTargetId,
) -> Result<()> {
    _ = diesel::delete(
        room_streaming_targets::table
            .filter(room_streaming_targets::id.eq(streaming_target_id))
            .filter(
                room_streaming_targets::room_id
                    .eq_any(rooms::table.select(rooms::id).filter_by_room(room)),
            ),
    )
    .execute(conn)
    .await?;

    Ok(())
}

/// Delete all streaming targets that are associated with a specific room
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn delete_by_room_id(conn: &mut DbConnection, room_id: RoomId) -> Result<()> {
    _ = diesel::delete(
        room_streaming_targets::table.filter(room_streaming_targets::room_id.eq(room_id)),
    )
    .execute(conn)
    .await?;

    Ok(())
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn insert(
    conn: &mut DbConnection,
    new_room_streaming_target: NewRoomStreamingTarget,
) -> Result<RoomStreamingTarget> {
    diesel::insert_into(room_streaming_targets::table)
        .values(new_room_streaming_target)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn update_room_streaming_target(
    conn: &mut DbConnection,
    update_room_streaming_target: UpdateRoomStreamingTarget,
    room: &RoomIdOrAlias,
    streaming_target_id: StreamingTargetId,
) -> Result<RoomStreamingTarget> {
    diesel::update(room_streaming_targets::table)
        .filter(room_streaming_targets::id.eq(streaming_target_id))
        .filter(
            room_streaming_targets::room_id
                .eq_any(rooms::table.select(rooms::id).filter_by_room(room)),
        )
        .set(update_room_streaming_target)
        .returning(room_streaming_targets::all_columns)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

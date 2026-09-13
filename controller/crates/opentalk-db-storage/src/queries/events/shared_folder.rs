// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use opentalk_database::{DatabaseError, DbConnection, Result};
use opentalk_types_common::{events::EventId, rooms::RoomId};

use crate::{
    schema::{event_shared_folders, events},
    tables::event_shared_folders::{EventSharedFolder, NewEventSharedFolder},
};

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_event_shared_folder(
    conn: &mut DbConnection,
    event_id: EventId,
) -> Result<Option<EventSharedFolder>> {
    event_shared_folders::table
        .filter(event_shared_folders::event_id.eq(event_id))
        .get_result(conn)
        .await
        .optional()
        .map_err(DatabaseError::from)
}

/// Returns all [`EventSharedFolder`]s in the given [`RoomId`].
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_event_shared_folders_for_room(
    conn: &mut DbConnection,
    room_id: RoomId,
) -> Result<Vec<EventSharedFolder>> {
    event_shared_folders::table
        .inner_join(events::table)
        .filter(events::room.eq(room_id))
        .select(event_shared_folders::all_columns)
        .load(conn)
        .await
        .map_err(DatabaseError::from)
}

/// Delete a shared folder using the given event id.
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn delete_shared_folder_by_event_id(
    conn: &mut DbConnection,
    event_id: EventId,
) -> Result<()> {
    diesel::delete(event_shared_folders::table.filter(event_shared_folders::event_id.eq(event_id)))
        .execute(conn)
        .await?;

    Ok(())
}

/// Delete shared folders using the given event ids
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn delete_shared_folders_by_event_ids(
    conn: &mut DbConnection,
    event_ids: &[EventId],
) -> Result<()> {
    diesel::delete(
        event_shared_folders::table.filter(event_shared_folders::event_id.eq_any(event_ids)),
    )
    .execute(conn)
    .await?;

    Ok(())
}

/// Tries to insert the EventSharedFolder into the database.
///
/// When yielding a unique constraint violation, None is returned.
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn try_create_event_shared_folder(
    conn: &mut DbConnection,
    new_shared_folder: NewEventSharedFolder,
) -> Result<Option<EventSharedFolder>> {
    let result = diesel::insert_into(event_shared_folders::table)
        .values(new_shared_folder)
        .get_result(conn)
        .await;

    match result {
        Ok(event_shared_folders) => Ok(Some(event_shared_folders)),
        Err(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            ..,
        )) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

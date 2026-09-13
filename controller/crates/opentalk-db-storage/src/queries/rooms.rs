// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains rooms database queries

use diesel::{dsl::not, prelude::*, query_builder::IntoUpdateTarget};
use diesel_async::RunQueryDsl;
use opentalk_database::{DatabaseError, DbConnection, Result};
use opentalk_types_common::{
    features::GUESTS_ALLOWED_MODULE_FEATURE_ID,
    pagination::{ItemCount, Page, PageSize},
    rooms::{GuestAccess, RoomAlias, RoomId, RoomIdOrAlias, RoomSuffix},
    users::UserId,
};

use crate::{
    self as db,
    paginate::Paginate,
    queries::room_filter::FilterByRoom,
    schema::{event_invites, events, rooms, tariffs, users},
    tables::{
        rooms::{NewRoom, Room, UpdateRoom},
        tariffs::Tariff,
        users::User,
    },
};

#[derive(diesel::Queryable)]
pub struct RoomAuthProperties {
    pub room_id: RoomId,
    pub created_by: UserId,
    pub guest_access: GuestAccess,
    pub e2e_encryption: bool,
    pub guests_allowed_by_tariff: bool,
}

const UNIQUE_SUFFIX_ATTEMPTS: u8 = 3;

/// Select a room using the given id or alias
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_room(conn: &mut DbConnection, room: &RoomIdOrAlias) -> Result<Room> {
    rooms::table
        .filter_by_room(room)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn exists_room(conn: &mut DbConnection, room: &RoomIdOrAlias) -> Result<bool> {
    diesel::select(diesel::dsl::exists(rooms::table.filter_by_room(room)))
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

/// Select a room and the creator using the given room id or alias
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_room_with_creator(
    conn: &mut DbConnection,
    room: &RoomIdOrAlias,
) -> Result<(Room, User)> {
    rooms::table
        .inner_join(users::table)
        .filter_by_room(room)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

/// Select all rooms joined with their creator
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_all_rooms_with_creator(conn: &mut DbConnection) -> Result<Vec<(Room, User)>> {
    rooms::table
        .order_by(rooms::id.desc())
        .inner_join(users::table)
        .load::<(Room, User)>(conn)
        .await
        .map_err(DatabaseError::from)
}

/// Select all rooms accessible to a certain user
#[tracing::instrument(err, skip_all)]
pub async fn get_accessible_to_user_with_creator_paginated(
    conn: &mut DbConnection,
    user: UserId,
    limit: PageSize,
    page: Page,
) -> Result<(Vec<(Room, User)>, ItemCount)> {
    let query = rooms::table
        .inner_join(users::table)
        .left_join(events::table)
        .left_join(event_invites::table.on(event_invites::event_id.eq(events::id)))
        .filter(
            rooms::created_by
                .eq(user)
                .or(event_invites::invitee.eq(user)),
        )
        .order_by(rooms::id.desc())
        .select((rooms::all_columns, users::all_columns))
        .paginate_by(limit, page);

    let rooms_with_total = query.load_and_count(conn).await?;

    Ok(rooms_with_total)
}

/// Select all rooms filtered by ids
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_by_ids_with_creator_paginated(
    conn: &mut DbConnection,
    ids: &[RoomId],
    limit: PageSize,
    page: Page,
) -> Result<(Vec<(Room, User)>, ItemCount)> {
    rooms::table
        .inner_join(users::table)
        .select((rooms::all_columns, users::all_columns))
        .filter(rooms::id.eq_any(ids))
        .order_by(rooms::id.desc())
        .paginate_by(limit, page)
        .load_and_count(conn)
        .await
        .map_err(DatabaseError::from)
}

/// Select all rooms that have no event associated with them
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_all_orphaned_room_ids(conn: &mut DbConnection) -> Result<Vec<RoomId>> {
    rooms::table
        .select(rooms::id)
        .filter(not(rooms::id.eq_any(events::table.select(events::room))))
        .order_by(rooms::created_at.asc())
        .load(conn)
        .await
        .map_err(DatabaseError::from)
}

/// Get the room's tariff
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_tariff(conn: &mut DbConnection, room: Room) -> Result<Tariff> {
    let user = db::queries::users::get_user(conn, room.created_by).await?;

    db::queries::tariffs::get_tariff(conn, user.tariff_id).await
}

/// Select all room ids with all properties relevant for authorization
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_all_room_ids_with_auth_properties(
    conn: &mut DbConnection,
) -> Result<Vec<RoomAuthProperties>> {
    rooms::table
        .inner_join(users::table.inner_join(tariffs::table))
        .select((
            rooms::id,
            rooms::created_by,
            rooms::guest_access,
            rooms::e2e_encryption,
            not(tariffs::disabled_features.contains(vec![GUESTS_ALLOWED_MODULE_FEATURE_ID])),
        ))
        .load(conn)
        .await
        .map_err(DatabaseError::from)
}

/// Delete a room using the given id
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn delete_room(conn: &mut DbConnection, room_id: RoomId) -> Result<()> {
    _ = diesel::delete(rooms::table.filter(rooms::id.eq(room_id)))
        .execute(conn)
        .await?;

    Ok(())
}

/// Create new room
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn create_room(conn: &mut DbConnection, mut new_room: NewRoom) -> Result<Room> {
    let mut attempts_left = UNIQUE_SUFFIX_ATTEMPTS;

    loop {
        match diesel::insert_into(rooms::table)
            .values(&new_room)
            .get_result(conn)
            .await
        {
            Ok(room) => return Ok(room),
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            )) if attempts_left > 1
                && let Some(suffix) = &mut new_room.suffix =>
            {
                attempts_left -= 1;
                let length = suffix.char_count();
                *suffix = RoomSuffix::generate(length);
            }
            // Out of retries or other error
            Err(e) => return Err(e.into()),
        }
    }
}

/// Update a room
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn update_room(
    conn: &mut DbConnection,
    update_room: UpdateRoom,
    room: &RoomIdOrAlias,
) -> Result<Room> {
    match room {
        RoomIdOrAlias::Id(id) => {
            update_room_with_suffix_retry(conn, update_room, rooms::table.filter(rooms::id.eq(id)))
                .await
        }
        RoomIdOrAlias::Alias(RoomAlias {
            name,
            suffix: Some(suffix),
        }) => {
            update_room_with_suffix_retry(
                conn,
                update_room,
                rooms::table
                    .filter(rooms::name.eq(name))
                    .filter(rooms::suffix.eq(suffix)),
            )
            .await
        }
        RoomIdOrAlias::Alias(RoomAlias { name, suffix: None }) => {
            update_room_with_suffix_retry(
                conn,
                update_room,
                rooms::table
                    .filter(rooms::name.eq(name))
                    .filter(rooms::suffix.is_null()),
            )
            .await
        }
    }
}

/// Runs a room `UPDATE`, regenerating the suffix and retrying on a unique-suffix collision.
async fn update_room_with_suffix_retry<T>(
    conn: &mut DbConnection,
    mut update_room: UpdateRoom,
    filter: T,
) -> Result<Room>
where
    T: IntoUpdateTarget<Table = rooms::table> + Copy,
    T::WhereClause: diesel::query_builder::QueryFragment<diesel::pg::Pg> + Send,
{
    let mut attempts_left = UNIQUE_SUFFIX_ATTEMPTS;
    loop {
        match diesel::update(filter)
            .set(&update_room)
            .get_result(conn)
            .await
        {
            Ok(room) => return Ok(room),
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            )) if attempts_left > 1
                && let Some(Some(suffix)) = &mut update_room.suffix =>
            {
                attempts_left -= 1;
                let length = suffix.char_count();
                *suffix = RoomSuffix::generate(length);
            }
            // Out of retries or other error
            Err(e) => return Err(e.into()),
        }
    }
}

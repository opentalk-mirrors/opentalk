// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::{ExpressionMethods as _, JoinOnDsl as _, OptionalExtension as _, QueryDsl as _};
use diesel_async::RunQueryDsl as _;
use opentalk_database::{DatabaseError, DbConnection, Result};
use opentalk_types_common::{events::invites::InviteRole, rooms::RoomIdOrAlias, users::UserId};

use crate::{
    queries::room_filter::FilterByRoom as _,
    schema::{event_invites, events, rooms, tariffs, users},
    tables::{rooms::Room, tariffs::Tariff},
};

pub async fn is_room_owner(
    conn: &mut DbConnection,
    room: &RoomIdOrAlias,
    user_id: UserId,
) -> Result<bool> {
    diesel::select(diesel::dsl::exists(
        rooms::table
            .filter_by_room(room)
            .filter(rooms::created_by.eq(user_id)),
    ))
    .get_result(conn)
    .await
    .map_err(DatabaseError::from)
}

pub async fn get_room_user_role(
    conn: &mut DbConnection,
    room: &RoomIdOrAlias,
    user_id: UserId,
) -> Result<Option<InviteRole>> {
    rooms::table
        .inner_join(events::table.on(events::room.eq(rooms::id)))
        .inner_join(event_invites::table.on(event_invites::event_id.eq(events::id)))
        .filter(event_invites::invitee.eq(user_id))
        .filter_by_room(room)
        .select(event_invites::role)
        .get_result(conn)
        .await
        .optional()
        .map_err(DatabaseError::from)
}

/// Returns the [`Room`], and [`Tariff`] for a given [`RoomIdOrAlias`].
///
/// Returns `Ok(None)` if the request succeeds but the `room` does not match an
/// existing entry, which is semantically clearer than a [`DatabaseError`].
pub async fn get_room_and_tariff(
    conn: &mut DbConnection,
    room: &RoomIdOrAlias,
) -> Result<Option<(Room, Tariff)>> {
    rooms::table
        .filter_by_room(room)
        .inner_join(users::table.on(users::id.eq(rooms::created_by)))
        .inner_join(tariffs::table.on(tariffs::id.eq(users::tariff_id)))
        .select((rooms::all_columns, tariffs::all_columns))
        .get_result(conn)
        .await
        .optional()
        .map_err(DatabaseError::from)
}

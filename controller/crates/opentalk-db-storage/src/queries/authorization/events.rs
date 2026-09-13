// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

// pub async fn get_event_user_role(conn: &mut DbConnection, )

use diesel::{ExpressionMethods as _, JoinOnDsl as _, OptionalExtension as _, QueryDsl as _};
use diesel_async::RunQueryDsl as _;
use opentalk_database::{DatabaseError, DbConnection, Result};
use opentalk_types_common::{
    events::{EventId, invites::InviteRole},
    users::UserId,
};

use crate::{
    schema::{event_invites, events, rooms, tariffs, users},
    tables::{rooms::Room, tariffs::Tariff},
};

pub async fn is_event_owner(
    conn: &mut DbConnection,
    event_id: EventId,
    user_id: UserId,
) -> Result<bool> {
    diesel::select(diesel::dsl::exists(
        events::table
            .filter(events::id.eq(event_id))
            .filter(events::created_by.eq(user_id)),
    ))
    .get_result(conn)
    .await
    .map_err(DatabaseError::from)
}

pub async fn get_event_user_role(
    conn: &mut DbConnection,
    event_id: EventId,
    user_id: UserId,
) -> Result<Option<InviteRole>> {
    events::table
        .filter(events::id.eq(event_id))
        .inner_join(event_invites::table.on(event_invites::event_id.eq(events::id)))
        .filter(event_invites::invitee.eq(user_id))
        .select(event_invites::role)
        .get_result(conn)
        .await
        .optional()
        .map_err(DatabaseError::from)
}

/// Returns the [`Room`], and [`Tariff`] for a given [`EventId`].
///
/// Returns `Ok(None)` if the request succeeds but the `event_id` or `invite_code` do not match an
/// existing entry, which is semantically clearer than a [`DatabaseError`].
pub async fn get_event_room_and_tariff(
    conn: &mut DbConnection,
    event_id: EventId,
) -> Result<Option<(Room, Tariff)>> {
    events::table
        .filter(events::id.eq(event_id))
        .inner_join(rooms::table.on(rooms::id.eq(events::room)))
        .inner_join(users::table.on(users::id.eq(events::created_by)))
        .inner_join(tariffs::table.on(tariffs::id.eq(users::tariff_id)))
        .select((rooms::all_columns, tariffs::all_columns))
        .get_result(conn)
        .await
        .optional()
        .map_err(DatabaseError::from)
}

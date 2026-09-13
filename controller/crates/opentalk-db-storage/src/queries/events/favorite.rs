// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::{BoolExpressionMethods, ExpressionMethods};
use diesel_async::RunQueryDsl;
use opentalk_database::{DbConnection, Result};
use opentalk_types_common::{events::EventId, users::UserId};

use crate::{
    schema::event_favorites,
    tables::event_favorites::{EventFavorite, NewEventFavorite},
};

/// Deletes a EventFavorite entry by user_id and event_id.
///
/// Returns true if something was deleted.
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn delete_event_favorite_for_user(
    conn: &mut DbConnection,
    user_id: UserId,
    event_id: EventId,
) -> Result<bool> {
    let lines_changes = diesel::delete(event_favorites::table)
        .filter(
            event_favorites::user_id
                .eq(user_id)
                .and(event_favorites::event_id.eq(event_id)),
        )
        .execute(conn)
        .await?;

    Ok(lines_changes > 0)
}

/// Tries to insert the NewEventFavorite into the database.
///
/// When yielding a unique key violation, None is returned.
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn try_create_event_favorite_for_user(
    conn: &mut DbConnection,
    new_event_favorite: NewEventFavorite,
) -> Result<Option<EventFavorite>> {
    let result = diesel::insert_into(event_favorites::table)
        .values(new_event_favorite)
        .get_result(conn)
        .await;

    match result {
        Ok(event_favorite) => Ok(Some(event_favorite)),
        Err(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            ..,
        )) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

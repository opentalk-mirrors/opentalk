// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use opentalk_database::{DatabaseError, DbConnection, Result};
use opentalk_types_common::{
    events::{EventId, invites::EventInviteStatus},
    pagination::{ItemCount, Page, PageSize},
    rooms::RoomId,
    users::UserId,
};

use crate::{
    self as db,
    paginate::Paginate,
    schema::{event_invites, events, users},
    tables::{
        event_invites::{EventInvite, NewEventInvite, UpdateEventInvite},
        events::Event,
        users::User,
    },
};

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_event_user_invites_for_events(
    conn: &mut DbConnection,
    events: &[&Event],
) -> Result<Vec<Vec<(EventInvite, User)>>> {
    conn.transaction(async |conn| {
        let invites: Vec<EventInvite> = EventInvite::belonging_to(events).load(conn).await?;
        let mut user_ids: Vec<UserId> = invites.iter().map(|x| x.invitee).collect();
        // Small optimization to filter out duplicates
        user_ids.sort_unstable();
        user_ids.dedup();

        let users = db::queries::users::get_users_by_ids(conn, &user_ids).await?;

        let invites_by_event: Vec<Vec<EventInvite>> = invites.grouped_by(events);
        let mut invites_with_users_by_event = Vec::with_capacity(events.len());

        for invites in invites_by_event {
            let mut invites_with_users = Vec::with_capacity(invites.len());

            for invite in invites {
                let user = users
                    .iter()
                    .find(|user| user.id == invite.invitee)
                    .ok_or_else(|| DatabaseError::Custom {
                        message: "bug: user invite invitee missing".to_owned(),
                    })?;

                invites_with_users.push((invite, user.clone()))
            }

            invites_with_users_by_event.push(invites_with_users);
        }

        Ok(invites_with_users_by_event)
    })
    .await
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_event_invites_paginated(
    conn: &mut DbConnection,
    event_id: EventId,
    per_page: PageSize,
    page: Page,
    filter_by_status: Option<EventInviteStatus>,
) -> Result<(Vec<(EventInvite, User)>, ItemCount)> {
    let allowed_states = filter_by_status
        .map(|s| BTreeSet::from([s]))
        .unwrap_or_else(EventInviteStatus::all_enum_values);

    event_invites::table
        .inner_join(users::table.on(event_invites::invitee.eq(users::id)))
        .filter(
            event_invites::columns::event_id
                .eq(event_id)
                .and(event_invites::columns::status.eq_any(allowed_states)),
        )
        .order(event_invites::created_at.desc())
        .then_order_by(event_invites::created_by.desc())
        .then_order_by(event_invites::invitee.desc())
        .paginate_by(per_page, page)
        .load_and_count(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_invites_pending_for_user(
    conn: &mut DbConnection,
    user_id: UserId,
) -> Result<Vec<EventInvite>> {
    event_invites::table
        .filter(
            event_invites::invitee
                .eq(user_id)
                .and(event_invites::status.eq(EventInviteStatus::Pending)),
        )
        .load(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_event_invite_for_user_and_room(
    conn: &mut DbConnection,
    user_id: UserId,
    room_id: RoomId,
) -> Result<Option<EventInvite>> {
    event_invites::table
        .select(event_invites::all_columns)
        .inner_join(
            events::table.on(events::id
                .eq(event_invites::event_id)
                .and(events::room.eq(room_id))),
        )
        .filter(event_invites::invitee.eq(user_id))
        .first(conn)
        .await
        .optional()
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn delete_event_invite_by_invitee(
    conn: &mut DbConnection,
    event_id: EventId,
    invitee: UserId,
) -> Result<EventInvite> {
    diesel::delete(event_invites::table)
        .filter(
            event_invites::event_id
                .eq(event_id)
                .and(event_invites::invitee.eq(invitee)),
        )
        .returning(event_invites::all_columns)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

/// Tries to insert the EventInvite into the database.
///
/// When yielding a unique key violation, None is returned.
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn try_create_event_invite(
    conn: &mut DbConnection,
    new_invite: NewEventInvite,
) -> Result<Option<EventInvite>> {
    let result = diesel::insert_into(event_invites::table)
        .values(new_invite)
        .get_result(conn)
        .await;

    match result {
        Ok(event_invite) => Ok(Some(event_invite)),
        Err(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            ..,
        )) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Apply the update to the invite where `user_id` is the invitee.
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn update_event_invite(
    conn: &mut DbConnection,
    event_id: EventId,
    user_id: UserId,
    updated_event_invite: UpdateEventInvite,
) -> Result<EventInvite> {
    // TODO: Check if the update actually applied a change (also have a look at fn `apply` of
    // `UpdateEventEmailInvite`)
    //
    // Use something like:
    //
    // UPDATE event_invites
    // SET status = $status
    // WHERE id = $id RETURNING id, status, (SELECT status FROM tmp WHERE id = $id);
    //
    // or
    //
    // UPDATE event_invites
    // SET status = $status
    // WHERE id = $id FROM event_invites old RETURNING old.*;
    //
    // and compare the value to the set one to return if the value was changed.
    diesel::update(event_invites::table)
        .filter(
            event_invites::event_id
                .eq(event_id)
                .and(event_invites::invitee.eq(user_id)),
        )
        .set(updated_event_invite)
        // change it here
        .returning(event_invites::all_columns)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

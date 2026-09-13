// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use opentalk_database::{DatabaseError, DbConnection, Result};
use opentalk_types_common::{
    events::EventId,
    pagination::{ItemCount, Page, PageSize},
    rooms::RoomId,
};

use crate::{
    paginate::Paginate as _,
    schema::{event_email_invites, event_invites, events},
    tables::{
        event_email_invites::{EventEmailInvite, NewEventEmailInvite, UpdateEventEmailInvite},
        event_invites::NewEventInvite,
        events::Event,
        users::User,
    },
};

pub async fn migrate_event_email_invites_to_user_invites(
    conn: &mut DbConnection,
    user: &User,
) -> Result<Vec<(EventId, RoomId)>> {
    conn.transaction(async |conn| {
        let email_invites_with_room: Vec<(EventEmailInvite, RoomId)> = event_email_invites::table
            .filter(event_email_invites::email.eq(&user.email))
            .inner_join(events::table)
            .filter(events::tenant_id.eq(user.tenant_id))
            .select((event_email_invites::all_columns, events::room))
            .load(conn)
            .await?;

        if email_invites_with_room.is_empty() {
            return Ok(vec![]);
        }

        let event_ids = email_invites_with_room
            .iter()
            .map(|(email_invite, room_id)| (email_invite.event_id, *room_id))
            .collect();

        let new_invites: Vec<_> = email_invites_with_room
            .into_iter()
            .map(|(email_invite, _)| NewEventInvite {
                event_id: email_invite.event_id,
                invitee: user.id,
                role: email_invite.role.into(),
                created_by: email_invite.created_by,
                created_at: Some(email_invite.created_at),
            })
            .collect();

        diesel::insert_into(event_invites::table)
            .values(new_invites)
            .on_conflict_do_nothing()
            .execute(conn)
            .await?;

        diesel::delete(
            event_email_invites::table.filter(
                event_email_invites::email.eq(&user.email).and(
                    event_email_invites::event_id.eq_any(
                        events::table
                            .filter(events::tenant_id.eq(user.tenant_id))
                            .select(events::id),
                    ),
                ),
            ),
        )
        .execute(conn)
        .await?;

        Ok(event_ids)
    })
    .await
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_event_email_invites_for_events(
    conn: &mut DbConnection,
    events: &[&Event],
) -> Result<Vec<Vec<EventEmailInvite>>> {
    let invites: Vec<EventEmailInvite> = EventEmailInvite::belonging_to(events).load(conn).await?;
    let invites_by_event: Vec<Vec<EventEmailInvite>> = invites.grouped_by(events);
    Ok(invites_by_event)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn delete_event_email_invite_by_email(
    conn: &mut DbConnection,
    event_id: &EventId,
    email: &str,
) -> Result<EventEmailInvite> {
    diesel::delete(event_email_invites::table)
        .filter(
            event_email_invites::event_id
                .eq(event_id)
                .and(event_email_invites::email.eq(email)),
        )
        .returning(event_email_invites::all_columns)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_event_email_invites_paginated(
    conn: &mut DbConnection,
    event_id: EventId,
    limit: PageSize,
    page: Page,
) -> Result<(Vec<EventEmailInvite>, ItemCount)> {
    let query = event_email_invites::table
        .filter(event_email_invites::columns::event_id.eq(event_id))
        .order(event_email_invites::created_at.desc())
        .then_order_by(event_email_invites::created_by.desc())
        .then_order_by(event_email_invites::email.desc())
        .paginate_by(limit, page);

    let invites: (Vec<EventEmailInvite>, ItemCount) = query.load_and_count(conn).await?;

    Ok(invites)
}

/// Tries to insert the EventEmailInvite into the database
///
/// When yielding a unique key violation, None is returned.
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn try_create_event_email_invite(
    conn: &mut DbConnection,
    new_email_invite: NewEventEmailInvite,
) -> Result<Option<EventEmailInvite>> {
    let result = diesel::insert_into(event_email_invites::table)
        .values(new_email_invite)
        .get_result(conn)
        .await;

    match result {
        Ok(event_email_invites) => Ok(Some(event_email_invites)),
        Err(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            ..,
        )) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Apply the update to the invite where `email` is the invitee's email address.
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn update_event_email_invite(
    conn: &mut DbConnection,
    email: &str,
    event_id: EventId,
    update_email_invite: UpdateEventEmailInvite,
) -> Result<EventEmailInvite> {
    // TODO: Check if the update actually applied a change (see comments in fn `apply` of
    // `UpdateEventInvite`)
    diesel::update(event_email_invites::table)
        .filter(
            event_email_invites::event_id
                .eq(event_id)
                .and(event_email_invites::email.eq(email)),
        )
        .set(update_email_invite)
        .returning(event_email_invites::all_columns)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

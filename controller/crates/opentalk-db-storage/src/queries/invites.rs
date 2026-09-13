// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains invites database queries

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use opentalk_database::{DatabaseError, DbConnection, Result};
use opentalk_types_common::{
    pagination::{ItemCount, Page, PageSize},
    rooms::{RoomId, invite_codes::InviteCode},
    users::UserId,
};

use crate::{
    paginate::Paginate as _,
    schema::{invites, users},
    tables::{invites::Invite, users::User},
};

pub type InviteWithUsers = (Invite, User, User);

/// Query for an invite with the given id
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_room_invite(
    conn: &mut DbConnection,
    invite_code_id: InviteCode,
) -> Result<Invite> {
    invites::table
        .filter(invites::id.eq(invite_code_id))
        .order(invites::updated_at.desc())
        .first(conn)
        .await
        .map_err(DatabaseError::from)
}

/// Retrieve all invites
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_all_invites(conn: &mut DbConnection) -> Result<Vec<Invite>> {
    invites::table.load(conn).await.map_err(DatabaseError::from)
}

/// Returns a invites with user metadata for id
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_room_invite_with_creator_and_updater(
    conn: &mut DbConnection,
    invite_code_id: InviteCode,
) -> Result<InviteWithUsers> {
    // Diesel currently does not support joining a table twice, so we need to join once and do a
    // second select. Or we need to write our handwritten SQL here.
    let query = invites::table
        .filter(invites::id.eq(invite_code_id))
        .inner_join(users::table.on(invites::created_by.eq(users::id)))
        .order(invites::updated_at.desc());
    let (invite, created_by) = query.first::<(Invite, User)>(conn).await?;

    let query = users::table.filter(users::id.eq(invite.updated_by));
    Ok((invite, created_by, query.first(conn).await?))
}

/// Returns a paginated view on invites for the given room
///
///
/// Returns:
/// Vec<(Invite, CreatedByUser, UpdatedByUser)> - A Vec of invites along with the users that created
/// and updated the invite
#[tracing::instrument(err(level = "debug"), skip_all, fields(%limit, %page))]
pub async fn get_all_for_room_paginated(
    conn: &mut DbConnection,
    room_id: RoomId,
    limit: PageSize,
    page: Page,
) -> Result<(Vec<Invite>, ItemCount)> {
    invites::table
        .filter(invites::room.eq(room_id))
        .order(invites::updated_at.desc())
        .paginate_by(limit, page)
        .load_and_count::<Invite, _>(conn)
        .await
        .map_err(DatabaseError::from)
}

/// Returns an active (enabled and not expired) invite for a given room, if there is any.
///
/// Note: this does not check whether guest access is actually permitted for the
/// room.
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_active_invite_for_room(
    conn: &mut DbConnection,
    room_id: RoomId,
    now: DateTime<Utc>,
) -> Result<Option<Invite>> {
    invites::table
        .filter(
            invites::room.eq(room_id).and(invites::active.eq(true)).and(
                invites::expiration
                    .is_null()
                    .or(invites::expiration.gt(now)),
            ),
        )
        .order(invites::updated_at.desc())
        .first::<Invite>(conn)
        .await
        .optional()
        .map_err(DatabaseError::from)
}

/// Returns a paginated view on invites for the given room
///
/// Returns:
/// Vec<(Invite, CreatedByUser, UpdatedByUser)> - A Vec of invites along with the users that created
/// and updated the invite
#[tracing::instrument(err(level = "debug"), skip_all, fields(%limit, %page))]
pub async fn get_room_invites_paginated_with_creator_and_updater(
    conn: &mut DbConnection,
    room_id: RoomId,
    limit: PageSize,
    page: Page,
) -> Result<(Vec<InviteWithUsers>, ItemCount)> {
    let query = invites::table
        .filter(invites::room.eq(room_id))
        .inner_join(users::table.on(invites::created_by.eq(users::id)))
        .order(invites::updated_at.desc())
        .paginate_by(limit, page);

    let (invites_with_user, total) = query.load_and_count::<(Invite, User), _>(conn).await?;

    // This needs urgent improvement, this will come up more times when we follow the created_by,
    // updated_by pattern.
    let users_set = invites_with_user
        .iter()
        .fold(HashSet::new(), |mut acc, (user, _)| {
            acc.insert(user.updated_by);
            acc
        });

    let users = users_set.iter().collect::<Vec<_>>();

    let query = users::table.filter(users::id.eq_any(users));
    let updated_by = query.get_results::<User>(conn).await?;
    let updated_by = updated_by
        .into_iter()
        .map(|u| (u.id, u))
        .collect::<HashMap<_, _>>();

    Ok((
        invites_with_user
            .into_iter()
            .map(|(invite, created_by)| {
                let updated_by_id = invite.updated_by;
                (
                    invite,
                    created_by,
                    updated_by
                        .get(&updated_by_id)
                        .expect("Some Foreign Key was wrong in our database")
                        .clone(),
                )
            })
            .collect::<Vec<_>>(),
        total,
    ))
}

/// Returns a paginated view on invites for the given room
///
/// Filters based on the passed user. Only invites are returned that where created or updated by the
/// passed in user.
///
/// Returns:
/// Vec<(Invite, CreatedByUser, UpdatedByUser)> - A Vec of invites along with the users that created
/// and updated the invite
// FIXME(r.floren): When diesel 2.0 gets release this can be reworked to use proper aliases
#[tracing::instrument(err(level = "debug"), skip_all, fields(%limit, %page))]
pub async fn get_all_invites_for_room_with_users_by_ids_paginated(
    conn: &mut DbConnection,
    room_id: RoomId,
    ids: &[InviteCode],
    limit: PageSize,
    page: Page,
) -> Result<(Vec<InviteWithUsers>, ItemCount)> {
    let query = invites::table
        .filter(invites::room.eq(room_id))
        .filter(invites::id.eq_any(ids))
        .inner_join(users::table.on(invites::created_by.eq(users::id)))
        .order(invites::updated_at.desc())
        .paginate_by(limit, page);

    let (invites_with_user, total) = query.load_and_count::<(Invite, User), _>(conn).await?;

    // This needs urgent improvement, this will come up more times when we follow the created_by,
    // updated_by pattern.
    let users_set = invites_with_user
        .iter()
        .fold(HashSet::new(), |mut acc, (user, _)| {
            acc.insert(user.updated_by);
            acc
        });
    let users = users_set.iter().collect::<Vec<_>>();

    let query = users::table.filter(users::id.eq_any(users));
    let updated_by = query.get_results::<User>(conn).await?;
    let updated_by = updated_by
        .into_iter()
        .map(|u| (u.id, u))
        .collect::<HashMap<_, _>>();

    Ok((
        invites_with_user
            .into_iter()
            .map(|(invite, created_by)| {
                let updated_by_id = invite.updated_by;
                (
                    invite,
                    created_by,
                    updated_by
                        .get(&updated_by_id)
                        .expect("Some Foreign Key was wrong in our database")
                        .clone(),
                )
            })
            .collect::<Vec<_>>(),
        total,
    ))
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_inactive_or_expired_before_invite(
    conn: &mut DbConnection,
    expiration_date: DateTime<Utc>,
) -> Result<Vec<(InviteCode, RoomId)>> {
    invites::table
        .filter(
            invites::active.eq(false).or(invites::expiration
                .is_not_null()
                .and(invites::expiration.lt(expiration_date))),
        )
        .select((invites::id, invites::room))
        .get_results(conn)
        .await
        .map_err(DatabaseError::from)
}

/// Query all invites that where updated by the specified user.
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_room_invites_updated_by(
    conn: &mut DbConnection,
    user_id: UserId,
) -> Result<Vec<Invite>> {
    invites::table
        .filter(invites::updated_by.eq(user_id))
        .load(conn)
        .await
        .map_err(DatabaseError::from)
}

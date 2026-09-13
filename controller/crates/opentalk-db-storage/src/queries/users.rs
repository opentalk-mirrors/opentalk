// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains users database queries

use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::{DateTime, Utc};
use diesel::{pg::Pg, prelude::*};
use diesel_async::RunQueryDsl;
use opentalk_database::{DatabaseError, DbConnection, Result};
use opentalk_types_common::{
    pagination::{ItemCount, Page, PageSize},
    tenants::TenantId,
    users::UserId,
};

use crate::{
    levenshtein, lower,
    paginate::Paginate,
    schema::{assets, room_assets, rooms, users},
    soundex,
    tables::users::{NewUser, UpdateUser, User},
};

const MAX_USER_SEARCH_RESULTS: usize = 50;

/// Helper function to get all active users
fn active_users_query() -> users::BoxedQuery<'static, Pg> {
    users::table
        .filter(users::disabled_since.is_null())
        .into_boxed()
}

/// Get an active user with the given `id`. If the user has a `disabled_since` entry, a NotFound error is returned.
///
/// If no user exists with `user_id` this returns an Error
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_user(conn: &mut DbConnection, user_id: UserId) -> Result<User> {
    active_users_query()
        .filter(users::id.eq(user_id))
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

/// Get a user with the given `id` inside a tenant
///
/// If no user exists with `user_id` this returns an Error
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_user_by_tenant(
    conn: &mut DbConnection,
    tenant_id: TenantId,
    user_id: UserId,
) -> Result<User> {
    active_users_query()
        .filter(users::id.eq(user_id))
        .filter(users::tenant_id.eq(tenant_id))
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

/// Get a user with the given id
///
/// Returns None if no user matches `email`
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_user_by_email(
    conn: &mut DbConnection,
    tenant_id: TenantId,
    email: &str,
) -> Result<Option<User>> {
    active_users_query()
        .filter(users::tenant_id.eq(tenant_id))
        .filter(users::email.eq(email))
        .get_result(conn)
        .await
        .optional()
        .map_err(DatabaseError::from)
}

/// Get all users paginated
#[tracing::instrument(err(level = "debug"), skip_all, fields(%limit, %page))]
pub async fn get_all_paginated(
    conn: &mut DbConnection,
    limit: PageSize,
    page: Page,
) -> Result<(Vec<User>, ItemCount)> {
    active_users_query()
        .order_by(users::id.desc())
        .paginate_by(limit, page)
        .load_and_count(conn)
        .await
        .map_err(DatabaseError::from)
}

/// Get all users
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_all_users(conn: &mut DbConnection) -> Result<Vec<User>> {
    users::table
        .get_results(conn)
        .await
        .map_err(DatabaseError::from)
}

/// Get Users paginated and filtered by ids
#[tracing::instrument(err(level = "debug"), skip_all, fields(%limit, %page))]
pub async fn get_by_ids_paginated(
    conn: &mut DbConnection,
    ids: &[UserId],
    limit: PageSize,
    page: Page,
) -> Result<(Vec<User>, ItemCount)> {
    active_users_query()
        .filter(users::id.eq_any(ids))
        .order_by(users::id.desc())
        .paginate_by(limit, page)
        .load_and_count::<User, _>(conn)
        .await
        .map_err(DatabaseError::from)
}

/// Returns all `User`s filtered by id
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_users_by_ids(conn: &mut DbConnection, ids: &[UserId]) -> Result<Vec<User>> {
    active_users_query()
        .filter(users::id.eq_any(ids))
        .load(conn)
        .await
        .map_err(DatabaseError::from)
}

/// Get all users filtered by the given subs
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_all_by_oidc_subs(
    conn: &mut DbConnection,
    tenant_id: TenantId,
    subs: &[&str],
) -> Result<Vec<User>> {
    active_users_query()
        .filter(users::tenant_id.eq(tenant_id))
        .filter(users::oidc_sub.eq_any(subs))
        .load(conn)
        .await
        .map_err(DatabaseError::from)
}

/// Find users by search string
///
/// This looks for similarities of the search_str in the display_name, first+lastname and email
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn find_users(
    conn: &mut DbConnection,
    tenant_id: TenantId,
    search_str: &str,
    max_users: usize,
) -> Result<Vec<User>> {
    // IMPORTANT: lowercase it to match the index of the db and
    // remove all existing % in name and to avoid manipulation of the LIKE query.
    let search_str = search_str.replace('%', "").trim().to_lowercase();

    if search_str.is_empty() {
        return Ok(vec![]);
    }

    let like_query = format!("%{search_str}%");

    let lower_display_name = lower(users::display_name);

    let lower_first_lastname = lower(users::firstname.concat(" ").concat(users::lastname));

    active_users_query()
        .filter(users::tenant_id.eq(tenant_id))
        .filter(
            // First try LIKE query on display_name
            lower_display_name.like(&like_query).or(
                // Then try LIKE query with first+last name
                lower_first_lastname
                    .like(&like_query)
                    // Then try LIKE query on email
                    .or(lower(users::email).like(&like_query))
                    //
                    // Then SOUNDEX on display_name
                    .or(soundex(lower_display_name)
                        .eq(soundex(&search_str))
                        // only take SOUNDEX results with a levenshtein score of lower than 5
                        .and(levenshtein(lower_display_name, &search_str).lt(5)))
                    //
                    // Then SOUNDEX on first+last name
                    .or(soundex(lower_first_lastname)
                        .eq(soundex(&search_str))
                        // only take SOUNDEX results with a levenshtein score of lower than 5
                        .and(levenshtein(lower_first_lastname, &search_str).lt(5))),
            ),
        )
        .order_by(levenshtein(lower_display_name, &search_str))
        .then_order_by(levenshtein(lower_first_lastname, &search_str))
        .then_order_by(users::id)
        .limit(MAX_USER_SEARCH_RESULTS.min(max_users) as i64)
        .load(conn)
        .await
        .map_err(DatabaseError::from)
}

pub async fn get_user_storage_used_size(
    conn: &mut DbConnection,
    user_id: &UserId,
) -> Result<BigDecimal> {
    assets::table
        .inner_join(room_assets::table.inner_join(rooms::table))
        .filter(rooms::created_by.eq(user_id))
        .select(diesel::dsl::sum(assets::size))
        .first(conn)
        .await
        .map(Option::unwrap_or_default)
        .map_err(DatabaseError::from)
}

pub async fn get_used_storage_used_size_u64(
    conn: &mut DbConnection,
    user_id: &UserId,
) -> Result<u64> {
    let used_storage = get_user_storage_used_size(conn, user_id).await?;

    Ok(used_storage.to_u64().unwrap_or_else(|| {
        tracing::warn!("failed to convert used storage: {used_storage} to u64");
        u64::MAX
    }))
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_user_ids_disabled_before(
    conn: &mut DbConnection,
    date: DateTime<Utc>,
) -> Result<Vec<UserId>> {
    users::table
        .select(users::id)
        .filter(users::disabled_since.le(date))
        .load(conn)
        .await
        .map_err(Into::into)
}

/// Delete a user using the given id
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn delete_user(conn: &mut DbConnection, user_id: UserId) -> Result<()> {
    let _ = diesel::delete(users::table.filter(users::id.eq(user_id)))
        .execute(conn)
        .await?;

    Ok(())
}

/// Updates the last_authenticated_ab flag using the given id
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn set_last_authenticated_at_to_now(
    conn: &mut DbConnection,
    user_id: UserId,
) -> Result<()> {
    let _ = diesel::update(users::table.filter(users::id.eq(user_id)))
        .set((
            users::last_authenticated_at.eq(diesel::dsl::now),
            users::updated_at.eq(diesel::dsl::now),
        ))
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn create_user(conn: &mut DbConnection, new_user: NewUser) -> Result<User> {
    diesel::insert_into(users::table)
        .values(new_user)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

pub async fn create_or_update_user_by_oidc_sub(
    conn: &mut DbConnection,
    new_user: NewUser,
    enforce_display_name_on_update: bool,
) -> Result<User> {
    let NewUser {
        oidc_sub: _,
        email,
        title,
        firstname,
        lastname,
        language: _,
        display_name,
        phone,
        tenant_id: _,
        tariff_id,
        tariff_status,
        avatar_url,
        timezone,
    } = new_user.clone();
    let update_user = UpdateUser {
        title: Some(&title),
        email: Some(email),
        firstname: Some(&firstname),
        lastname: Some(&lastname),
        phone: Some(phone),
        display_name: enforce_display_name_on_update.then_some(&display_name),
        language: None,
        dashboard_theme: None,
        conference_theme: None,
        tariff_id: Some(tariff_id),
        tariff_status: Some(tariff_status),
        disabled_since: None,
        avatar_url: Some(avatar_url.as_deref()),
        timezone: Some(timezone),
        updated_at: Utc::now(),
    };

    new_user
        .insert_into(users::table)
        .on_conflict((users::oidc_sub, users::tenant_id))
        .do_update()
        .set(update_user)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

pub async fn update_user(
    conn: &mut DbConnection,
    update_user: UpdateUser<'_>,
    user_id: UserId,
) -> Result<User> {
    diesel::update(users::table.filter(users::id.eq(user_id)))
        .set(update_user)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

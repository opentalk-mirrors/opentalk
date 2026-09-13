// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::assert_matches;

use opentalk_database::DatabaseError;
use opentalk_db_storage::{
    self as db,
    tables::{
        rooms::{NewRoom, UpdateRoom},
        users::User,
    },
};
use opentalk_types_common::{
    rooms::{GuestAccess, RoomName, RoomSuffix},
    utils::ExampleData as _,
};
use pretty_assertions::{assert_eq, assert_ne};

use crate::common::make_user;

mod common;

fn new_room(user: &User, name: Option<RoomName>, suffix: Option<RoomSuffix>) -> NewRoom {
    NewRoom {
        created_by: user.id,
        password: None,
        waiting_room: false,
        guest_access: GuestAccess::default(),
        e2e_encryption: false,
        tenant_id: user.tenant_id,
        name,
        suffix,
    }
}

/// Build an [`UpdateRoom`] that only touches the alias (`name` and `suffix`) and
/// leaves every other field untouched.
fn update_alias(name: Option<RoomName>, suffix: Option<RoomSuffix>) -> UpdateRoom {
    UpdateRoom {
        name: Some(name),
        suffix: Some(suffix),
        password: None,
        waiting_room: None,
        guest_access: None,
        e2e_encryption: None,
    }
}

#[tokio::test]
async fn update_room_persists_name_and_suffix() {
    let db_ctx = opentalk_test_util::database::DatabaseContext::new().await;
    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let user = make_user(&mut conn, "Ruth", "Less", "Ruth").await;

    // Start from a room without an alias, then assign one via `update_room`.
    let room = db::queries::rooms::create_room(&mut conn, new_room(&user, None, None))
        .await
        .unwrap();

    let name = RoomName::example_data();
    let suffix = RoomSuffix::generate(16);

    let updated = db::queries::rooms::update_room(
        &mut conn,
        update_alias(Some(name.clone()), Some(suffix.clone())),
        &room.id.into(),
    )
    .await
    .unwrap();

    assert_eq!(updated.id, room.id);
    assert_eq!(updated.name, Some(name));
    assert_eq!(updated.suffix, Some(suffix));
}

#[tokio::test]
async fn update_room_regenerates_suffix_on_collision() {
    let db_ctx = opentalk_test_util::database::DatabaseContext::new().await;
    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let user = make_user(&mut conn, "Ruth", "Less", "Ruth").await;

    let name = RoomName::example_data();
    let suffix = RoomSuffix::generate(16);

    // The first room occupies the alias `(name, suffix)`.
    let first = db::queries::rooms::create_room(
        &mut conn,
        new_room(&user, Some(name.clone()), Some(suffix.clone())),
    )
    .await
    .unwrap();

    // A second, aliasless room that we will update into a collision.
    let second = db::queries::rooms::create_room(&mut conn, new_room(&user, None, None))
        .await
        .unwrap();

    // Updating the second room to the exact same alias must detect the unique
    // violation, regenerate the suffix and still succeed.
    let updated = db::queries::rooms::update_room(
        &mut conn,
        update_alias(Some(name.clone()), Some(suffix.clone())),
        &second.id.into(),
    )
    .await
    .unwrap();

    assert_eq!(updated.id, second.id);
    assert_eq!(updated.name, Some(name.clone()));
    // The suffix must have been regenerated to resolve the collision.
    assert_ne!(updated.suffix, Some(suffix.clone()));

    // The first room keeps its original alias.
    assert_eq!(first.name, Some(name));
    assert_eq!(first.suffix, Some(suffix));
}

#[tokio::test]
async fn update_room_fails_on_collision_without_suffix() {
    let db_ctx = opentalk_test_util::database::DatabaseContext::new().await;
    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let user = make_user(&mut conn, "Ruth", "Less", "Ruth").await;

    let name = RoomName::example_data();

    // The first room occupies the name-only alias `(name, NULL)`.
    let first =
        db::queries::rooms::create_room(&mut conn, new_room(&user, Some(name.clone()), None))
            .await
            .unwrap();

    assert_eq!(first.name, Some(name.clone()));
    assert_eq!(first.suffix, None);

    // A second, aliasless room that we will update into a collision.
    let second = db::queries::rooms::create_room(&mut conn, new_room(&user, None, None))
        .await
        .unwrap();

    // Updating the second room to the same name without a suffix cannot be resolved by regenerating the suffix, so it
    // must fail.
    let err = db::queries::rooms::update_room(
        &mut conn,
        update_alias(Some(name), None),
        &second.id.into(),
    )
    .await
    .unwrap_err();

    assert_matches!(
        err,
        DatabaseError::DieselError {
            source: diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _
            )
        }
    );
}

#[tokio::test]
async fn update_room_allows_multiple_rooms_without_alias() {
    let db_ctx = opentalk_test_util::database::DatabaseContext::new().await;
    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let user = make_user(&mut conn, "Ruth", "Less", "Ruth").await;

    let name = RoomName::example_data();

    // Two rooms that start with distinct aliases.
    let first = db::queries::rooms::create_room(
        &mut conn,
        new_room(&user, Some(name.clone()), Some(RoomSuffix::generate(16))),
    )
    .await
    .unwrap();
    let second = db::queries::rooms::create_room(
        &mut conn,
        new_room(&user, Some(name), Some(RoomSuffix::generate(16))),
    )
    .await
    .unwrap();

    // Removing the alias yields a `NULL` alias, which may exist any number of times, so both updates succeed.
    let first_updated =
        db::queries::rooms::update_room(&mut conn, update_alias(None, None), &first.id.into())
            .await
            .unwrap();
    let second_updated =
        db::queries::rooms::update_room(&mut conn, update_alias(None, None), &second.id.into())
            .await
            .unwrap();

    assert_eq!(first_updated.name, None);
    assert_eq!(first_updated.suffix, None);
    assert_eq!(second_updated.name, None);
    assert_eq!(second_updated.suffix, None);
    assert_ne!(first_updated.id, second_updated.id);
}

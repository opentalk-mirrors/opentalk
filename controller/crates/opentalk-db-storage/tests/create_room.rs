// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::assert_matches;

use opentalk_database::DatabaseError;
use opentalk_db_storage::{self as db};
use opentalk_types_common::{
    rooms::{RoomName, RoomSuffix},
    utils::ExampleData as _,
};
use pretty_assertions::{assert_eq, assert_ne};

use crate::common::{make_user, new_room};

mod common;

#[tokio::test]
async fn create_room_persists_name_and_suffix() {
    let db_ctx = opentalk_test_util::database::DatabaseContext::new().await;
    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let user = make_user(&mut conn, "Ruth", "Less", "Ruth").await;

    let name = RoomName::example_data();
    let suffix = RoomSuffix::generate(16);

    let room = db::queries::rooms::create_room(
        &mut conn,
        new_room(&user, Some(name.clone()), Some(suffix.clone())),
    )
    .await
    .unwrap();

    assert_eq!(room.name, Some(name));
    assert_eq!(room.suffix, Some(suffix));
    assert_eq!(room.created_by, user.id);
}

#[tokio::test]
async fn create_room_regenerates_suffix_on_collision() {
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

    // The second room requests the exact same alias. `create_room` must detect
    // the unique violation, regenerate the suffix and still succeed.
    let second = db::queries::rooms::create_room(
        &mut conn,
        new_room(&user, Some(name.clone()), Some(suffix.clone())),
    )
    .await
    .unwrap();

    assert_eq!(first.name, Some(name.clone()));
    assert_eq!(second.name, Some(name));
    assert_eq!(first.suffix, Some(suffix.clone()));
    // The suffix must have been regenerated to resolve the collision.
    assert_ne!(second.suffix, Some(suffix));
    assert_ne!(second.id, first.id);
}

#[tokio::test]
async fn create_room_fails_on_collision_without_suffix() {
    let db_ctx = opentalk_test_util::database::DatabaseContext::new().await;
    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let user = make_user(&mut conn, "Ruth", "Less", "Ruth").await;

    let name = RoomName::example_data();

    let room =
        db::queries::rooms::create_room(&mut conn, new_room(&user, Some(name.clone()), None))
            .await
            .unwrap();

    assert_eq!(room.name, Some(name.clone()));
    assert_eq!(room.suffix, None);

    // Trying to create another room with the same name and no suffix fails
    let err = db::queries::rooms::create_room(&mut conn, new_room(&user, Some(name), None))
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
async fn create_room_allows_multiple_rooms_without_alias() {
    let db_ctx = opentalk_test_util::database::DatabaseContext::new().await;
    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let user = make_user(&mut conn, "Ruth", "Less", "Ruth").await;

    // Rooms without a name have a `NULL` alias and may exist any number of times.
    let first = db::queries::rooms::create_room(&mut conn, new_room(&user, None, None))
        .await
        .unwrap();
    let second = db::queries::rooms::create_room(&mut conn, new_room(&user, None, None))
        .await
        .unwrap();

    assert_eq!(first.name, None);
    assert_eq!(first.suffix, None);
    assert_eq!(second.name, None);
    assert_ne!(first.id, second.id);
}

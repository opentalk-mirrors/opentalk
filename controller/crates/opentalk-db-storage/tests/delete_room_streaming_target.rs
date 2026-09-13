// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage as db;
use opentalk_types_common::{
    rooms::{RoomAlias, RoomName, RoomSuffix},
    utils::ExampleData as _,
};
use pretty_assertions::assert_eq;

use crate::common::{STREAMING_TARGET_NAME, create_streaming_target, make_user, new_room};

mod common;

const SUFFIX_LENGTH: u8 = 8;

#[tokio::test]
async fn delete_room_streaming_target_by_id_removes_the_target() {
    let db_ctx = opentalk_test_util::database::DatabaseContext::new().await;
    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let user = make_user(&mut conn, "Ruth", "Less", "Ruth").await;

    let room = db::queries::rooms::create_room(&mut conn, new_room(&user, None, None))
        .await
        .unwrap();
    let target_id = create_streaming_target(&mut conn, room.id, STREAMING_TARGET_NAME).await;

    db::queries::streaming_targets::delete_room_streaming_target(
        &mut conn,
        &room.id.into(),
        target_id,
    )
    .await
    .unwrap();

    let err =
        db::queries::streaming_targets::get_room_streaming_target(&mut conn, target_id, room.id)
            .await
            .unwrap_err();
    assert!(err.is_not_found());
}

#[tokio::test]
async fn delete_room_streaming_target_by_alias_removes_the_target() {
    let db_ctx = opentalk_test_util::database::DatabaseContext::new().await;
    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let user = make_user(&mut conn, "Ruth", "Less", "Ruth").await;

    let name = RoomName::example_data();
    let suffix = RoomSuffix::generate(SUFFIX_LENGTH);
    let room = db::queries::rooms::create_room(
        &mut conn,
        new_room(&user, Some(name.clone()), Some(suffix.clone())),
    )
    .await
    .unwrap();
    let target_id = create_streaming_target(&mut conn, room.id, STREAMING_TARGET_NAME).await;

    db::queries::streaming_targets::delete_room_streaming_target(
        &mut conn,
        &RoomAlias {
            name,
            suffix: Some(suffix),
        }
        .into(),
        target_id,
    )
    .await
    .unwrap();

    let err =
        db::queries::streaming_targets::get_room_streaming_target(&mut conn, target_id, room.id)
            .await
            .unwrap_err();
    assert!(err.is_not_found());
}

#[tokio::test]
async fn delete_room_streaming_target_only_affects_the_matching_room() {
    let db_ctx = opentalk_test_util::database::DatabaseContext::new().await;
    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let user = make_user(&mut conn, "Ruth", "Less", "Ruth").await;

    let room = db::queries::rooms::create_room(&mut conn, new_room(&user, None, None))
        .await
        .unwrap();
    let other = db::queries::rooms::create_room(&mut conn, new_room(&user, None, None))
        .await
        .unwrap();
    let target_id = create_streaming_target(&mut conn, room.id, STREAMING_TARGET_NAME).await;

    // Deleting the target while filtering by the wrong room must not remove it.
    db::queries::streaming_targets::delete_room_streaming_target(
        &mut conn,
        &other.id.into(),
        target_id,
    )
    .await
    .unwrap();

    let fetched =
        db::queries::streaming_targets::get_room_streaming_target(&mut conn, target_id, room.id)
            .await
            .unwrap();
    assert_eq!(fetched.id, target_id);
}

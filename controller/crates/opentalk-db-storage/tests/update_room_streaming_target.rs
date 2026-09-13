// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage::{self as db, tables::room_streaming_targets::UpdateRoomStreamingTarget};
use opentalk_types_common::{
    rooms::{RoomAlias, RoomName, RoomSuffix},
    utils::ExampleData as _,
};
use pretty_assertions::assert_eq;

use crate::common::{STREAMING_TARGET_NAME, create_streaming_target, make_user, new_room};

mod common;

const SUFFIX_LENGTH: u8 = 8;

/// Name the streaming target is renamed to during update tests.
const UPDATED_STREAMING_TARGET_NAME: &str = "Renamed";

fn rename(name: &str) -> UpdateRoomStreamingTarget {
    UpdateRoomStreamingTarget {
        name: Some(name.to_owned()),
        kind: None,
        streaming_endpoint: None,
        streaming_key: None,
        public_url: None,
    }
}

#[tokio::test]
async fn update_room_streaming_target_by_id_updates_the_target() {
    let db_ctx = opentalk_test_util::database::DatabaseContext::new().await;
    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let user = make_user(&mut conn, "Ruth", "Less", "Ruth").await;

    let room = db::queries::rooms::create_room(&mut conn, new_room(&user, None, None))
        .await
        .unwrap();
    let target_id = create_streaming_target(&mut conn, room.id, STREAMING_TARGET_NAME).await;

    let updated = db::queries::streaming_targets::update_room_streaming_target(
        &mut conn,
        rename(UPDATED_STREAMING_TARGET_NAME),
        &room.id.into(),
        target_id,
    )
    .await
    .unwrap();

    assert_eq!(updated.id, target_id);
    assert_eq!(updated.room_id, room.id);
    assert_eq!(updated.name, UPDATED_STREAMING_TARGET_NAME);

    // The change is persisted.
    let fetched =
        db::queries::streaming_targets::get_room_streaming_target(&mut conn, target_id, room.id)
            .await
            .unwrap();
    assert_eq!(fetched.name, UPDATED_STREAMING_TARGET_NAME);
}

#[tokio::test]
async fn update_room_streaming_target_by_alias_updates_the_target() {
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

    let updated = db::queries::streaming_targets::update_room_streaming_target(
        &mut conn,
        rename(UPDATED_STREAMING_TARGET_NAME),
        &RoomAlias {
            name,
            suffix: Some(suffix),
        }
        .into(),
        target_id,
    )
    .await
    .unwrap();

    assert_eq!(updated.id, target_id);
    assert_eq!(updated.name, UPDATED_STREAMING_TARGET_NAME);
}

#[tokio::test]
async fn update_room_streaming_target_from_wrong_room_returns_not_found() {
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

    let err = db::queries::streaming_targets::update_room_streaming_target(
        &mut conn,
        rename(UPDATED_STREAMING_TARGET_NAME),
        &other.id.into(),
        target_id,
    )
    .await
    .unwrap_err();
    assert!(err.is_not_found());

    // The target keeps its original name.
    let fetched =
        db::queries::streaming_targets::get_room_streaming_target(&mut conn, target_id, room.id)
            .await
            .unwrap();
    assert_eq!(fetched.name, STREAMING_TARGET_NAME);
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage as db;
use pretty_assertions::assert_eq;

use crate::common::{STREAMING_TARGET_NAME, create_streaming_target, make_user, new_room};

mod common;

#[tokio::test]
async fn get_room_streaming_target_by_id_returns_the_target() {
    let db_ctx = opentalk_test_util::database::DatabaseContext::new().await;
    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let user = make_user(&mut conn, "Ruth", "Less", "Ruth").await;

    let room = db::queries::rooms::create_room(&mut conn, new_room(&user, None, None))
        .await
        .unwrap();
    let target_id = create_streaming_target(&mut conn, room.id, STREAMING_TARGET_NAME).await;

    let fetched =
        db::queries::streaming_targets::get_room_streaming_target(&mut conn, target_id, room.id)
            .await
            .unwrap();

    assert_eq!(fetched.id, target_id);
    assert_eq!(fetched.room_id, room.id);
    assert_eq!(fetched.name, STREAMING_TARGET_NAME);
}

#[tokio::test]
async fn get_room_streaming_target_from_wrong_room_returns_not_found() {
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

    // The streaming target exists, but not in `other`, so it must not be returned.
    let err =
        db::queries::streaming_targets::get_room_streaming_target(&mut conn, target_id, other.id)
            .await
            .unwrap_err();

    assert!(err.is_not_found());
}

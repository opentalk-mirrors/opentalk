// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage::{self as db, tables::sip_configs::NewSipConfig};
use opentalk_types_common::{
    call_in::{CallInId, CallInPassword},
    rooms::{RoomAlias, RoomId, RoomName, RoomSuffix},
    utils::ExampleData as _,
};
use pretty_assertions::assert_eq;

use crate::common::{make_user, new_room};

mod common;

const SUFFIX_LENGTH: u8 = 8;

fn new_sip_config(room_id: RoomId) -> NewSipConfig {
    NewSipConfig {
        room: room_id,
        sip_id: CallInId::generate(),
        password: CallInPassword::generate(),
        enable_lobby: false,
    }
}

#[tokio::test]
async fn delete_room_sip_config_by_id_removes_the_config() {
    let db_ctx = opentalk_test_util::database::DatabaseContext::new().await;
    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let user = make_user(&mut conn, "Ruth", "Less", "Ruth").await;

    let room = db::queries::rooms::create_room(&mut conn, new_room(&user, None, None))
        .await
        .unwrap();
    db::queries::sip_configs::create_room_sip_config(&mut conn, new_sip_config(room.id))
        .await
        .unwrap();

    // The config exists before deletion.
    let fetched = db::queries::sip_configs::get_room_sip_config(&mut conn, room.id)
        .await
        .unwrap();
    assert_eq!(fetched.room, room.id);

    db::queries::sip_configs::delete_room_sip_config(&mut conn, &room.id.into())
        .await
        .unwrap();

    // After deletion the lookup reports the config as not found.
    let err = db::queries::sip_configs::get_room_sip_config(&mut conn, room.id)
        .await
        .unwrap_err();
    assert!(err.is_not_found());
}

#[tokio::test]
async fn delete_room_sip_config_by_alias_removes_the_config() {
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
    db::queries::sip_configs::create_room_sip_config(&mut conn, new_sip_config(room.id))
        .await
        .unwrap();

    db::queries::sip_configs::delete_room_sip_config(
        &mut conn,
        &RoomAlias {
            name,
            suffix: Some(suffix),
        }
        .into(),
    )
    .await
    .unwrap();

    let err = db::queries::sip_configs::get_room_sip_config(&mut conn, room.id)
        .await
        .unwrap_err();
    assert!(err.is_not_found());
}

#[tokio::test]
async fn delete_room_sip_config_only_deletes_the_matching_room() {
    let db_ctx = opentalk_test_util::database::DatabaseContext::new().await;
    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let user = make_user(&mut conn, "Ruth", "Less", "Ruth").await;

    let target = db::queries::rooms::create_room(&mut conn, new_room(&user, None, None))
        .await
        .unwrap();
    db::queries::sip_configs::create_room_sip_config(&mut conn, new_sip_config(target.id))
        .await
        .unwrap();

    let other = db::queries::rooms::create_room(&mut conn, new_room(&user, None, None))
        .await
        .unwrap();
    db::queries::sip_configs::create_room_sip_config(&mut conn, new_sip_config(other.id))
        .await
        .unwrap();

    db::queries::sip_configs::delete_room_sip_config(&mut conn, &target.id.into())
        .await
        .unwrap();

    // The target's config is gone.
    let err = db::queries::sip_configs::get_room_sip_config(&mut conn, target.id)
        .await
        .unwrap_err();
    assert!(err.is_not_found());

    // The unrelated room's config must remain untouched.
    let remaining = db::queries::sip_configs::get_room_sip_config(&mut conn, other.id)
        .await
        .unwrap();
    assert_eq!(remaining.room, other.id);
}

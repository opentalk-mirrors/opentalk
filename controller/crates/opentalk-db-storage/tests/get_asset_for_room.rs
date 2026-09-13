// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_db_storage::{
    self as db,
    tables::{assets::NewAsset, users::User},
};
use opentalk_types_common::{
    assets::AssetId,
    rooms::{RoomAlias, RoomName, RoomSuffix},
    utils::ExampleData as _,
};
use pretty_assertions::assert_eq;

use crate::common::{make_user, new_room};

mod common;

const SUFFIX_LENGTH: u8 = 8;

fn new_asset(user: &User) -> NewAsset {
    NewAsset {
        id: AssetId::generate(),
        namespace: None,
        kind: "test".into(),
        filename: "test.txt".into(),
        tenant_id: user.tenant_id,
        size: 42,
    }
}

#[tokio::test]
async fn get_asset_for_room_by_id_returns_the_asset() {
    let db_ctx = opentalk_test_util::database::DatabaseContext::new().await;
    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let user = make_user(&mut conn, "Ruth", "Less", "Ruth").await;

    let room = db::queries::rooms::create_room(&mut conn, new_room(&user, None, None))
        .await
        .unwrap();

    let created = db::queries::assets::create_asset_for_room(&mut conn, new_asset(&user), room.id)
        .await
        .unwrap();

    let fetched = db::queries::assets::get_asset_for_room(&mut conn, &room.id.into(), created.id)
        .await
        .unwrap();

    assert_eq!(fetched.id, created.id);
}

#[tokio::test]
async fn get_asset_for_room_by_alias_returns_the_asset() {
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

    let created = db::queries::assets::create_asset_for_room(&mut conn, new_asset(&user), room.id)
        .await
        .unwrap();

    let fetched = db::queries::assets::get_asset_for_room(
        &mut conn,
        &RoomAlias {
            name,
            suffix: Some(suffix),
        }
        .into(),
        created.id,
    )
    .await
    .unwrap();

    assert_eq!(fetched.id, created.id);
}

/// The room-asset association is the permission check: an asset must only be retrievable through a room it belongs to.
/// Requesting it via a different room's id must not leak the asset.
#[tokio::test]
async fn get_asset_for_room_by_wrong_room_id_returns_not_found() {
    let db_ctx = opentalk_test_util::database::DatabaseContext::new().await;
    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let user = make_user(&mut conn, "Ruth", "Less", "Ruth").await;

    let owning_room = db::queries::rooms::create_room(&mut conn, new_room(&user, None, None))
        .await
        .unwrap();

    let other_room = db::queries::rooms::create_room(&mut conn, new_room(&user, None, None))
        .await
        .unwrap();

    let created =
        db::queries::assets::create_asset_for_room(&mut conn, new_asset(&user), owning_room.id)
            .await
            .unwrap();

    let err = db::queries::assets::get_asset_for_room(&mut conn, &other_room.id.into(), created.id)
        .await
        .unwrap_err();

    assert!(err.is_not_found());
}

#[tokio::test]
async fn get_asset_for_room_by_wrong_room_alias_returns_not_found() {
    let db_ctx = opentalk_test_util::database::DatabaseContext::new().await;
    let mut conn = db_ctx.db.get_conn().await.unwrap();

    let user = make_user(&mut conn, "Ruth", "Less", "Ruth").await;

    let name = RoomName::example_data();
    let owning_room = db::queries::rooms::create_room(
        &mut conn,
        new_room(
            &user,
            Some(name.clone()),
            Some(RoomSuffix::generate(SUFFIX_LENGTH)),
        ),
    )
    .await
    .unwrap();

    // Create a second room with the same name but a different suffix
    let other_suffix = RoomSuffix::generate(SUFFIX_LENGTH);
    db::queries::rooms::create_room(
        &mut conn,
        new_room(&user, Some(name.clone()), Some(other_suffix.clone())),
    )
    .await
    .unwrap();

    let created =
        db::queries::assets::create_asset_for_room(&mut conn, new_asset(&user), owning_room.id)
            .await
            .unwrap();

    let err = db::queries::assets::get_asset_for_room(
        &mut conn,
        &RoomAlias {
            name,
            suffix: Some(other_suffix),
        }
        .into(),
        created.id,
    )
    .await
    .unwrap_err();

    assert!(err.is_not_found());
}

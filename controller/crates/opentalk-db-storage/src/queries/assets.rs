// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains assets database queries

use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl as _, NullableExpressionMethods, QueryDsl,
};
use diesel_async::{AsyncConnection, RunQueryDsl};
use opentalk_database::{DatabaseError, DbConnection, Result};
use opentalk_types_common::{
    assets::{AssetId, AssetSorting, FileSize},
    events::EventId,
    order::Ordering,
    pagination::{ItemCount, Page, PageSize},
    rooms::{RoomId, RoomIdOrAlias},
    users::UserId,
};

pub use crate::tables::assets::{Asset, NewAsset, UpdateAsset};
use crate::{
    paginate::Paginate as _,
    queries::room_filter::FilterByRoom,
    schema::{assets, events, room_assets, rooms},
    tables::assets::RoomAsset,
};

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_asset_for_room(
    conn: &mut DbConnection,
    room: &RoomIdOrAlias,
    asset_id: AssetId,
) -> Result<Asset> {
    //FIXME: The inner_joins below (as well as the room parameter) can be removed when assets have their own
    // permission check and don't rely on room permissions
    assets::table
        .inner_join(room_assets::table.on(room_assets::asset_id.eq(assets::id)))
        .inner_join(rooms::table.on(rooms::id.eq(room_assets::room_id)))
        .filter(assets::id.eq(asset_id))
        .select(assets::all_columns)
        .filter_by_room(room)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_all_asset_ids_for_room(
    conn: &mut DbConnection,
    room_id: RoomId,
) -> Result<Vec<AssetId>> {
    room_assets::table
        .select(room_assets::asset_id)
        .filter(room_assets::room_id.eq(room_id))
        .load(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn count_all_assets(conn: &mut DbConnection) -> Result<i64> {
    assets::table
        .count()
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_all_assets_for_room_paginated(
    conn: &mut DbConnection,
    room: &RoomIdOrAlias,
    limit: PageSize,
    page: Page,
) -> Result<(Vec<Asset>, ItemCount)> {
    assets::table
        .inner_join(room_assets::table.on(room_assets::asset_id.eq(assets::id)))
        .inner_join(rooms::table.on(rooms::id.eq(room_assets::room_id)))
        .select(assets::all_columns)
        .filter_by_room(room)
        .paginate_by(limit, page)
        .load_and_count(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_all_assets_for_rooms_paginated(
    conn: &mut DbConnection,
    room_ids: &[RoomId],
    limit: PageSize,
    page: Page,
) -> Result<(Vec<Asset>, ItemCount)> {
    assets::table
        .inner_join(room_assets::table.on(room_assets::asset_id.eq(assets::id)))
        .filter(room_assets::room_id.eq_any(room_ids))
        .select(assets::all_columns)
        .paginate_by(limit, page)
        .load_and_count(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_all_assets_with_size(conn: &mut DbConnection) -> Result<Vec<(AssetId, i64)>> {
    assets::table
        .select((assets::id, assets::size))
        .order_by(assets::created_at.asc())
        .load(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn delete_asset_from_room(
    conn: &mut DbConnection,
    room_id: RoomId,
    asset_id: AssetId,
) -> Result<FileSize> {
    conn.transaction(async |conn| {
        //FIXME: This check (as well as the room_id parameter) can be removed when assets have their own permission
        // check and don't rely on room permissions
        //
        // check if the asset exists for the specified room
        room_assets::table
            .filter(
                room_assets::asset_id
                    .eq(asset_id)
                    .and(room_assets::room_id.eq(room_id)),
            )
            .execute(conn)
            .await?;

        diesel::delete(assets::table.filter(assets::id.eq(asset_id)))
            .load::<Asset>(conn)
            .await?
            .iter()
            .map(|asset| asset.size)
            .reduce(|acc, e| acc.saturating_add(e))
            .ok_or(DatabaseError::NotFound)
    })
    .await
}

/// Used for the internal deletion of assets
///
/// When the request originates from a client, the [`delete_assets_by_ids`] method should be used.
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn delete_asset_by_id_internal(
    conn: &mut DbConnection,
    asset_id: &AssetId,
) -> Result<()> {
    let _ = diesel::delete(assets::table.filter(assets::id.eq(asset_id)))
        .execute(conn)
        .await?;

    Ok(())
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn delete_assets_by_ids(conn: &mut DbConnection, asset_ids: &[AssetId]) -> Result<()> {
    let _ = diesel::delete(assets::table.filter(assets::id.eq_any(asset_ids)))
        .execute(conn)
        .await?;

    Ok(())
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn create_asset_for_room(
    conn: &mut DbConnection,
    new_asset: NewAsset,
    room_id: RoomId,
) -> Result<Asset> {
    conn.transaction(async |conn| {
        let asset: Asset = diesel::insert_into(assets::table)
            .values(new_asset)
            .get_result(conn)
            .await?;

        {
            let room_asset = RoomAsset {
                room_id,
                asset_id: asset.id,
            };

            diesel::insert_into(room_assets::table)
                .values(room_asset)
                .execute(conn)
                .await?;
        }

        Ok(asset)
    })
    .await
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn update_asset(
    conn: &mut DbConnection,
    update_asset: UpdateAsset,
    asset_id: AssetId,
) -> Result<Asset> {
    let target = assets::table.filter(assets::id.eq(&asset_id));

    diesel::update(target)
        .set(update_asset)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

pub type AssetRoomIdEventIdTuple = (Asset, RoomId, Option<EventId>);

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_all_assets_for_room_owner_paginated_ordered(
    conn: &mut DbConnection,
    user_id: UserId,
    limit: PageSize,
    page: Page,
    sorting: AssetSorting,
    order: Ordering,
) -> Result<(Vec<AssetRoomIdEventIdTuple>, ItemCount)> {
    let mut query = room_assets::table
        .inner_join(assets::table)
        .inner_join(rooms::table.left_join(events::table))
        .filter(rooms::created_by.eq(user_id))
        .select((
            assets::all_columns,
            rooms::columns::id,
            events::columns::id.nullable(),
        ))
        .into_boxed();

    // There was no sane approach to move this block to it's own function or call asc/desc on a
    // generalized column.
    query = match (order, sorting) {
        (Ordering::Ascending, AssetSorting::Filename) => query.order(assets::filename.asc()),
        (Ordering::Ascending, AssetSorting::Size) => query.order(assets::size.asc()),
        (Ordering::Ascending, AssetSorting::Namespace) => query.order(assets::namespace.asc()),
        (Ordering::Ascending, AssetSorting::Kind) => query.order(assets::kind.asc()),
        (Ordering::Ascending, AssetSorting::CreatedAt) => query.order(assets::created_at.asc()),

        (Ordering::Descending, AssetSorting::Filename) => query.order(assets::filename.desc()),
        (Ordering::Descending, AssetSorting::Size) => query.order(assets::size.desc()),
        (Ordering::Descending, AssetSorting::Namespace) => query.order(assets::namespace.desc()),
        (Ordering::Descending, AssetSorting::Kind) => query.order(assets::kind.desc()),
        (Ordering::Descending, AssetSorting::CreatedAt) => query.order(assets::created_at.desc()),
    };

    let query = query.paginate_by(limit, page);
    Ok(query.load_and_count(conn).await?)
}

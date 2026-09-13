// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    assets::{AssetId, AssetSorting, FileSize},
    events::EventId,
    order::Ordering,
    pagination::{ItemCount, Page, PageSize},
    rooms::{RoomId, RoomIdOrAlias},
    users::UserId,
};

use super::{Asset, NewAsset, UpdateAsset};
use crate::Result;

/// A trait for retrieving and storing event entities.
#[async_trait::async_trait]
pub trait AssetInventory {
    /// Create an asset for a room.
    async fn create_asset_for_room(&mut self, room_id: RoomId, asset: NewAsset) -> Result<Asset>;

    /// Delete an asset from a room.
    async fn delete_asset_from_room(
        &mut self,
        room_id: RoomId,
        asset_id: AssetId,
    ) -> Result<FileSize>;

    /// Get an asset for a room.
    async fn get_asset_for_room(
        &mut self,
        room: &RoomIdOrAlias,
        asset_id: AssetId,
    ) -> Result<Asset>;

    /// Get all asset ids and their size
    async fn get_all_assets_with_size(&mut self) -> Result<Vec<(AssetId, i64)>>;

    /// Get all assets for a room, paginated.
    ///
    /// Returns a tuple of the loaded assets, and the overall count of available assets.
    async fn get_all_assets_for_room_paginated(
        &mut self,
        room: &RoomIdOrAlias,
        per_page: PageSize,
        page: Page,
    ) -> Result<(Vec<Asset>, ItemCount)>;

    /// Get all asset ids for a room.
    async fn get_all_asset_ids_for_room(&mut self, room_id: RoomId) -> Result<Vec<AssetId>>;

    /// Get all assets associated with rooms owned by a specific user, paginated and ordered.
    ///
    /// Returns a tuple of the loaded assets, and the overall count of available assets.
    async fn get_all_assets_for_room_owner_paginated_ordered(
        &mut self,
        user_id: UserId,
        limit: PageSize,
        page: Page,
        sort: AssetSorting,
        order: Ordering,
    ) -> Result<(Vec<(Asset, RoomId, Option<EventId>)>, ItemCount)>;

    /// Update an asset.
    ///
    /// Will return `Ok(None)` if no asset with the given `asset_id` is found.
    async fn update_asset(
        &mut self,
        asset_id: AssetId,
        asset: UpdateAsset,
    ) -> Result<Option<Asset>>;

    /// Internal deletion of assets.
    async fn delete_asset_by_id_internal(&mut self, asset_id: AssetId) -> Result<()>;

    /// Internal deletion of assets.
    async fn delete_assets_by_ids(&mut self, asset_ids: &[AssetId]) -> Result<()>;
}

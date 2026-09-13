// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use bytes::Bytes;
use futures_core::Stream;
use opentalk_asset_storage::{
    AssetError, AssetSaved, ByStreamExt, ChunkFormat, NewAssetFileName, ObjectStorageError,
    StorageNotifier, asset_key, delete_asset, get_asset, save_asset,
};
use opentalk_controller_utils::CaptureApiError;
use opentalk_types_api_internal::module_assets::AssetResource;
use opentalk_types_api_v1::{
    error::ApiError, pagination::PagePaginationQuery,
    rooms::by_room_id::assets::RoomsByRoomIdAssetsGetResponseBody,
};
use opentalk_types_common::{
    assets::AssetId, modules::ModuleId, pagination::ItemCount, rooms::RoomIdOrAlias,
};

use crate::{ControllerBackend, helpers::asset_to_asset_resource};

impl ControllerBackend {
    pub(crate) async fn get_room_assets(
        &self,
        room_id_or_alias: RoomIdOrAlias,
        pagination: &PagePaginationQuery,
    ) -> Result<(RoomsByRoomIdAssetsGetResponseBody, ItemCount), CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let (assets, asset_count) = inventory
            .get_all_assets_for_room_paginated(
                &room_id_or_alias,
                pagination.per_page,
                pagination.page,
            )
            .await?;

        let assets = assets.into_iter().map(asset_to_asset_resource).collect();
        let assets = RoomsByRoomIdAssetsGetResponseBody(assets);

        Ok((assets, asset_count))
    }

    pub(crate) async fn get_room_asset(
        &self,
        room_id_or_alias: RoomIdOrAlias,
        asset_id: AssetId,
    ) -> Result<ByStreamExt, CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let asset = inventory
            .get_asset_for_room(&room_id_or_alias, asset_id)
            .await?;

        let stream = get_asset(&self.storage, &asset.id).await?;

        Ok(stream)
    }

    pub(crate) async fn get_room_asset_proxy_download_token(
        &self,
        room_id_or_alias: RoomIdOrAlias,
        asset_id: AssetId,
    ) -> Result<String, CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;
        let asset = inventory
            .get_asset_for_room(&room_id_or_alias, asset_id)
            .await?;

        let encoded_filename = percent_encoding::utf8_percent_encode(
            &asset.filename,
            percent_encoding::NON_ALPHANUMERIC,
        )
        .to_string();

        let content_disposition = format!(
            "attachment; filename=\"{}\"; filename*=UTF-8''{}",
            asset.filename, encoded_filename
        );

        let token = self
            .storage
            .get_proxy_download_token(
                &asset_key(&asset.id),
                Duration::from_secs(30),
                content_disposition,
            )
            .await?;

        Ok(token)
    }

    #[tracing::instrument(level = "debug", skip(self, storage_notifier, data))]
    pub(crate) async fn create_room_asset(
        &self,
        storage_notifier: &dyn StorageNotifier,
        room_id_or_alias: RoomIdOrAlias,
        filename: NewAssetFileName,
        namespace: Option<ModuleId>,
        data: Box<dyn Stream<Item = Result<Bytes, ObjectStorageError>> + Unpin>,
    ) -> Result<(AssetResource, AssetSaved), CaptureApiError> {
        let settings = self.settings_provider.get();

        let res = save_asset(
            &self.storage.clone(),
            self.inventory_provider.as_ref(),
            storage_notifier,
            &room_id_or_alias,
            namespace,
            filename,
            data,
            ChunkFormat::Data,
            Some(settings.http.upload_size_limit),
        )
        .await;

        let asset_saved = match res {
            Err(AssetError::AssetStorageExceeded) => {
                return Err(CaptureApiError(
                    ApiError::bad_request().with_message("Asset storage exceeded"),
                ));
            }
            Err(e) => {
                return Err(e.into());
            }
            Ok(asset) => asset,
        };

        let asset = self
            .inventory_provider
            .get_inventory()
            .await?
            .get_asset_for_room(&room_id_or_alias, asset_saved.asset_id)
            .await?;

        Ok((asset_to_asset_resource(asset), asset_saved))
    }

    pub(crate) async fn delete_room_asset(
        &self,
        storage_notifier: &dyn StorageNotifier,
        room_id_or_alias: RoomIdOrAlias,
        asset_id: AssetId,
    ) -> Result<(), CaptureApiError> {
        delete_asset(
            &self.storage,
            self.inventory_provider.as_ref(),
            storage_notifier,
            &room_id_or_alias,
            asset_id,
        )
        .await?;

        Ok(())
    }
}

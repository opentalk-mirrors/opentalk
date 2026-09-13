// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::sync::Arc;

use futures::TryStreamExt;
use opentalk_asset_storage::{
    AssetError, ChunkFormat, ObjectStorage, ObjectStorageError, StorageNotifier, save_asset,
    verify_storage_usage,
};
use opentalk_controller_service_facade::NewAssetFileName;
use opentalk_controller_settings::SettingsProvider;
use opentalk_inventory::InventoryProvider;
use opentalk_roomserver_room::storage::{
    StorageContext,
    assets::{
        AssetMetaData, AssetStorageProvider, AssetStream, AssetUploaded, StorageError, UploadResult,
    },
};
use opentalk_types_api_internal::module_assets::Quota;
use opentalk_types_common::users::UserId;

#[derive(Debug)]
pub(crate) struct AssetStorage {
    settings_provider: SettingsProvider,
    inventory_provider: Arc<dyn InventoryProvider>,
    storage: Arc<ObjectStorage>,
    storage_notifier: Arc<dyn StorageNotifier>,
    room_owner: UserId,
}

impl AssetStorage {
    pub fn new(
        settings_provider: SettingsProvider,
        inventory_provider: Arc<dyn InventoryProvider>,
        storage: Arc<ObjectStorage>,
        storage_notifier: Arc<dyn StorageNotifier>,
        room_owner: UserId,
    ) -> Self {
        Self {
            settings_provider,
            inventory_provider,
            storage,
            storage_notifier,
            room_owner,
        }
    }
}

#[async_trait::async_trait]
impl AssetStorageProvider for AssetStorage {
    async fn upload_asset(
        &self,
        asset: AssetStream,
        metadata: AssetMetaData,
        context: &StorageContext,
    ) -> UploadResult {
        let filename = NewAssetFileName::new_with_event_title(
            context.event.as_ref().map(|event| event.title.clone()),
            metadata.kind,
            metadata.timestamp,
            metadata.extension,
        );
        let settings = self.settings_provider.get();
        let data = asset.map_err(|e| ObjectStorageError::Other {
            message: "Upload error".to_string(),
            source: Some(e.into()),
        });

        save_asset(
            &self.storage,
            self.inventory_provider.as_ref(),
            self.storage_notifier.as_ref(),
            &context.room_id.into(),
            Some(context.namespace.clone()),
            filename,
            data,
            ChunkFormat::Data,
            Some(settings.http.upload_size_limit),
        )
        .await
        .map(|asset_saved| AssetUploaded {
            id: asset_saved.asset_id,
            filename: asset_saved.filename,
            quota: asset_saved.quota,
        })
        .map_err(|err| match err {
            AssetError::InventoryConnection { source } | AssetError::InventoryQuery { source } => {
                StorageError::internal(source)
            }
            AssetError::ObjectStorage { source } => StorageError::internal(source),
            AssetError::FileSize { source } => StorageError::internal(source),
            AssetError::AssetStorageExceeded => StorageError::QuotaExceeded,
            AssetError::Rollback {
                source,
                rollback_reason: _,
            } => StorageError::internal(source),
        })
    }

    async fn can_upload(&self) -> bool {
        let Ok(mut inventory) = self.inventory_provider.get_inventory().await else {
            log::error!("Failed to get inventory");
            return false;
        };

        verify_storage_usage(&mut *inventory, self.room_owner)
            .await
            .is_ok()
    }

    // The internal storage does not need to be updated,
    // it is already up-to-date when storage notifications are received.
    async fn set_storage_quota(&self, _: Quota) {}
}

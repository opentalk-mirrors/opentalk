// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

pub mod controller_asset_storage;
pub mod controller_module_storage;
pub mod memory_asset_storage;
pub mod memory_module_storage;

pub use opentalk_roomserver_signaling::storage::StorageContext;

pub mod assets {
    pub use opentalk_roomserver_signaling::storage::assets::{
        AssetMetaData, AssetUploaded, ModuleAssetStorage, StorageError, UploadResult,
        provider::{AssetStorageProvider, AssetStream},
    };
}

pub mod module_resources {
    pub use opentalk_roomserver_signaling::storage::module_resources::{
        Error, provider::ModuleResourceProvider,
    };
}

// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{any::Any, fmt::Debug};

use async_trait::async_trait;
use bytes::Bytes;
use futures::stream::BoxStream;
use opentalk_types_api_internal::module_assets::Quota;

use crate::storage::assets::{AssetMetaData, StorageContext, UploadResult};

pub type AssetStream = BoxStream<'static, Result<Bytes, AssetLoadError>>;

#[derive(Debug)]
pub struct AssetLoadError {
    pub source: Box<dyn std::error::Error + Send + Sync>,
}

impl<E> From<E> for AssetLoadError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(value: E) -> Self {
        Self {
            source: Box::new(value),
        }
    }
}

// this is is required to transform the AssetStream into a reqwest Body.
impl From<AssetLoadError> for Box<dyn std::error::Error + Send + Sync> {
    fn from(err: AssetLoadError) -> Self {
        err.source
    }
}

#[async_trait]
pub trait AssetStorageProvider: Send + Sync + Debug + Any {
    /// Uploads an asset to the storage backend
    async fn upload_asset(
        &self,
        asset: AssetStream,
        metadata: AssetMetaData,
        context: &StorageContext,
    ) -> UploadResult;

    async fn can_upload(&self) -> bool;

    async fn set_storage_quota(&self, quota: Quota);
}

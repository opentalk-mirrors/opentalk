// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use anyhow::Context as _;
use async_trait::async_trait;
use opentalk_roomserver_signaling::storage::{
    StorageContext,
    assets::{
        AssetMetaData, AssetUploaded, StorageError, UploadResult,
        provider::{AssetStorageProvider, AssetStream},
    },
};
use opentalk_service_auth::ApiKey;
use opentalk_types_api_internal::{
    error::ApiError,
    module_assets::{PostAssetResponseBody, Quota},
};
use reqwest::{Body, header::CONTENT_TYPE};
use tokio::sync::RwLock;
use url::Url;

const MAX_BODY_PREVIEW_LENGTH: usize = 80;

/// `ControllerAssetStorage` is an implementation of the [`AssetStorageProvider`] trait that
/// interacts with a remote controller service to manage asset uploads and storage quota for a
/// roomserver.
#[derive(Debug)]
pub struct ControllerAssetStorage {
    base_url: Url,
    secret: ApiKey,
    client: reqwest::Client,
    quota: RwLock<Quota>,
}

impl ControllerAssetStorage {
    pub fn new(base_url: Url, secret: ApiKey, quota: Quota) -> Self {
        Self {
            base_url,
            secret,
            client: reqwest::Client::new(),
            quota: RwLock::new(quota),
        }
    }
}

#[async_trait]
impl AssetStorageProvider for ControllerAssetStorage {
    async fn upload_asset(
        &self,
        asset: AssetStream,
        metadata: AssetMetaData,
        context: &StorageContext,
    ) -> UploadResult {
        let mut query = vec![
            ("namespace", context.namespace.as_str()),
            ("file_extension", metadata.extension.as_str()),
            ("kind", metadata.kind.as_str()),
        ];
        if let Some(event_title) = context.event.as_ref().map(|e| e.title.as_str()) {
            query.push(("event_title", event_title));
        }

        let token = self
            .secret
            .generate_jwt()
            .context("Failed to generate auth header")?;

        let response = self
            .client
            .post(
                self.base_url
                    .join(&format!("/internal/room/{}/asset", context.room_id))
                    .context("Invalid URL")?,
            )
            .bearer_auth(token)
            .body(Body::wrap_stream(asset))
            .header(CONTENT_TYPE, "application/octet-stream")
            .query(&query)
            .send()
            .await
            .context("Asset upload request failed")?;

        let status = response.status();
        let body = response
            .bytes()
            .await
            .context("Failed to read upload asset response body")?;

        if status == ApiError::storage_quota_exceeded().status {
            tracing::debug!("Asset upload failed due to exceeded storage quota");
            return Err(StorageError::QuotaExceeded);
        }
        if !status.is_success() {
            let body_str = match std::str::from_utf8(&body) {
                Ok(s) if s.len() > MAX_BODY_PREVIEW_LENGTH => {
                    format!("{}...[truncated]", &s[..MAX_BODY_PREVIEW_LENGTH])
                }
                Ok(s) => s.to_string(),
                Err(_) => "<non-utf8>".to_string(),
            };
            tracing::error!(
                "Asset upload failed with status: `{}` Body: `{}`",
                status,
                body_str,
            );
            return Err(
                anyhow::anyhow!("Failed to upload asset due to unexpected server error").into(),
            );
        }

        let PostAssetResponseBody {
            asset_resource,
            quota,
        } = serde_json::from_slice(&body)
            .inspect_err(|e| {
                tracing::error!(
                    "Failed to parse upload response: {e:?} Body: {:?}",
                    std::str::from_utf8(&body).unwrap_or("<non-utf8>")
                );
            })
            .context("Failed to parse upload asset response body")?;

        *self.quota.write().await = quota.clone();

        Ok(AssetUploaded {
            id: asset_resource.id,
            filename: asset_resource.filename,
            quota,
        })
    }

    async fn can_upload(&self) -> bool {
        !self.quota.read().await.is_exceeded()
    }

    async fn set_storage_quota(&self, quota: Quota) {
        *self.quota.write().await = quota;
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use anyhow::anyhow;
    use bytes::Bytes;
    use futures::{StreamExt, stream};
    use mockito::Matcher;
    use opentalk_roomserver_crypto_provider::ensure_crypto_provider;
    use opentalk_roomserver_signaling::storage::{
        StorageContext,
        assets::{
            AssetMetaData, AssetUploaded, StorageError,
            provider::{AssetLoadError, AssetStorageProvider},
        },
    };
    use opentalk_service_auth::ApiKey;
    use opentalk_types_api_internal::{
        error::ApiError,
        module_assets::{AssetResource, PostAssetResponseBody, Quota},
    };
    use opentalk_types_common::{
        assets::{AssetFileKind, AssetId, FileExtension},
        modules::ModuleId,
        rooms::RoomId,
        time::Timestamp,
        utils::ExampleData,
    };
    use pretty_assertions::assert_eq;

    use super::ControllerAssetStorage;

    #[test_log::test(tokio::test)]
    async fn successful_asset_upload() {
        ensure_crypto_provider();

        let room_id = RoomId::from_u128(0x01);
        let asset_file_kind = AssetFileKind::example_data();
        let created_at = Timestamp::unix_epoch();
        let module_id = ModuleId::example_data();
        let response_body = PostAssetResponseBody {
            asset_resource: AssetResource {
                id: AssetId::from_u128(0x01),
                filename: "Testfile".to_string(),
                namespace: Some(module_id.clone()),
                created_at: *created_at,
                kind: asset_file_kind.to_string(),
                size: 12.into(),
            },
            quota: Quota {
                total: None,
                used: 12,
            },
        };

        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", format!("/internal/room/{room_id}/asset").as_str())
            .match_header("authorization", Matcher::Regex("^Bearer .*".into()))
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("namespace".to_string(), module_id.to_string()),
                mockito::Matcher::UrlEncoded(
                    "file_extension".to_string(),
                    FileExtension::pdf().to_string(),
                ),
                mockito::Matcher::UrlEncoded("kind".to_string(), asset_file_kind.to_string()),
            ]))
            // response
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_body(serde_json::to_string(&response_body).unwrap())
            .create_async()
            .await;

        let provider = ControllerAssetStorage::new(
            server.url().parse().unwrap(),
            ApiKey::new("controller", "secret"),
            Quota {
                total: None,
                used: 0,
            },
        );
        let asset_stream =
            stream::once(async { Ok(Bytes::copy_from_slice(r"asdasd".as_bytes())) }).boxed();

        let upload_result = provider
            .upload_asset(
                asset_stream,
                AssetMetaData {
                    kind: asset_file_kind,
                    timestamp: created_at,
                    extension: FileExtension::pdf(),
                },
                &StorageContext {
                    room_id,
                    namespace: ModuleId::example_data(),
                    event: None,
                },
            )
            .await;

        mock.assert_async().await;

        assert_eq!(
            upload_result.unwrap(),
            AssetUploaded {
                id: response_body.asset_resource.id,
                filename: response_body.asset_resource.filename,
                quota: response_body.quota,
            }
        );
    }

    #[test_log::test(tokio::test)]
    async fn quota_exceeded_asset_upload() {
        ensure_crypto_provider();

        let room_id = RoomId::from_u128(0x01);
        let asset_file_kind = AssetFileKind::example_data();
        let created_at = Timestamp::unix_epoch();
        let module_id = ModuleId::example_data();

        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", format!("/internal/room/{room_id}/asset").as_str())
            .match_header("authorization", Matcher::Regex("^Bearer .*".into()))
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("namespace".to_string(), module_id.to_string()),
                mockito::Matcher::UrlEncoded(
                    "file_extension".to_string(),
                    FileExtension::pdf().to_string(),
                ),
                mockito::Matcher::UrlEncoded("kind".to_string(), asset_file_kind.to_string()),
            ]))
            // response
            .with_status(ApiError::storage_quota_exceeded().status.as_u16() as usize)
            .create_async()
            .await;

        let provider = ControllerAssetStorage::new(
            server.url().parse().unwrap(),
            ApiKey::new("controller", "secret"),
            Quota {
                total: None,
                used: 0,
            },
        );
        let asset_stream =
            stream::once(async { Ok(Bytes::copy_from_slice(r"asdasd".as_bytes())) }).boxed();

        let upload_result = provider
            .upload_asset(
                asset_stream,
                AssetMetaData {
                    kind: asset_file_kind,
                    timestamp: created_at,
                    extension: FileExtension::pdf(),
                },
                &StorageContext {
                    room_id,
                    namespace: ModuleId::example_data(),
                    event: None,
                },
            )
            .await;

        mock.assert_async().await;

        assert_matches!(upload_result, Err(StorageError::QuotaExceeded));
    }

    #[test_log::test(tokio::test)]
    async fn test_asset_load_error() {
        ensure_crypto_provider();

        let room_id = RoomId::from_u128(0x01);
        let asset_file_kind = AssetFileKind::example_data();
        let created_at = Timestamp::unix_epoch();
        let module_id = ModuleId::example_data();

        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", format!("/internal/room/{room_id}/asset").as_str())
            .match_header("authorization", Matcher::Regex("^Bearer .*".into()))
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("namespace".to_string(), module_id.to_string()),
                mockito::Matcher::UrlEncoded(
                    "file_extension".to_string(),
                    FileExtension::pdf().to_string(),
                ),
                mockito::Matcher::UrlEncoded("kind".to_string(), asset_file_kind.to_string()),
            ]))
            // response
            .with_status(ApiError::storage_quota_exceeded().status.as_u16() as usize)
            .create_async()
            .await;

        let provider = ControllerAssetStorage::new(
            server.url().parse().unwrap(),
            ApiKey::new("controller", "secret"),
            Quota::example_data(),
        );

        let asset_stream = stream::iter(vec![Err(AssetLoadError {
            source: anyhow!("Error").into(),
        })])
        .boxed();
        let upload_result = provider
            .upload_asset(
                asset_stream,
                AssetMetaData {
                    kind: asset_file_kind,
                    timestamp: created_at,
                    extension: FileExtension::pdf(),
                },
                &StorageContext {
                    room_id,
                    namespace: ModuleId::example_data(),
                    event: None,
                },
            )
            .await;

        assert_matches!(upload_result, Err(StorageError::Internal(_)));
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use anyhow::anyhow;
use futures::{StreamExt, TryStreamExt as _};
use opentalk_roomserver_signaling::storage::assets::{
    AssetMetaData, AssetUploaded, ModuleAssetStorage, provider::AssetLoadError,
};
use opentalk_roomserver_types::{
    room_kind::RoomKind,
    signaling::module_error::{FatalError, SignalingModuleError},
};
use opentalk_roomserver_types_whiteboard::{WhiteboardError, state::SpaceInfo};
use opentalk_types_common::{
    assets::{AssetFileKind, FileExtension, asset_file_kind},
    rooms::RoomId,
    time::Timestamp,
};

use crate::client::SpacedeckClient;

pub enum WhiteboardLoopback {
    SpaceCreated { info: SpaceInfo },
    PdfCreated { asset: AssetUploaded },
}

#[tracing::instrument(skip(client), level = "debug")]
pub(super) async fn create_space(
    client: Arc<SpacedeckClient>,
    room_id: RoomId,
    room: RoomKind,
) -> Result<WhiteboardLoopback, SignalingModuleError<WhiteboardError>> {
    let name = build_space_name(room_id, room);
    let response = client.create_space(&name, None).await.map_err(|err| {
        tracing::error!("Failed to create space: {err}");
        WhiteboardError::InitializationFailed
    })?;

    let url = client
        .base_url
        .join(&format!(
            "s/{hash}-{slug}",
            hash = response.edit_hash,
            slug = response.edit_slug
        ))
        .map_err(|err| FatalError(anyhow!("Failed to build spacedeck URL: {err}")))?;

    Ok(WhiteboardLoopback::SpaceCreated {
        info: SpaceInfo {
            id: response.id,
            url,
        },
    })
}

#[tracing::instrument(skip_all, fields(id), level = "debug")]
pub(super) async fn generate_pdf(
    spacedeck_client: Arc<SpacedeckClient>,
    storage_client: ModuleAssetStorage,
    id: String,
    timestamp: Timestamp,
) -> Result<WhiteboardLoopback, SignalingModuleError<WhiteboardError>> {
    const ASSET_FILE_KIND: AssetFileKind = asset_file_kind!("whiteboard_pdf");

    tracing::debug!("Generating PDF for space '{id}'");
    let url = spacedeck_client.get_pdf(&id).await?;
    let stream = spacedeck_client
        .download_pdf(url)
        .await?
        .map_err(|e| AssetLoadError {
            source: Box::new(e),
        });

    let metadata = AssetMetaData {
        kind: ASSET_FILE_KIND,
        timestamp,
        extension: FileExtension::pdf(),
    };
    let asset = storage_client
        .upload_asset(stream.boxed(), metadata)
        .await
        .map_err(WhiteboardError::from)?;

    Ok(WhiteboardLoopback::PdfCreated { asset })
}

#[tracing::instrument(level = "debug", skip_all, fields(id))]
pub(super) async fn delete_space(
    spacedeck_client: Arc<SpacedeckClient>,
    storage_client: ModuleAssetStorage,
    id: String,
    timestamp: Timestamp,
) {
    // Generate a pdf so the state does not get lost
    if let Err(err) = generate_pdf(
        Arc::clone(&spacedeck_client),
        storage_client,
        id.clone(),
        timestamp,
    )
    .await
    {
        tracing::error!("Failed to generate PDF for space '{id}': {err:?}");
    }

    if let Err(err) = spacedeck_client.delete_space(&id).await {
        tracing::error!("Failed to delete space '{id}' from spacedeck: {err}");
    } else {
        tracing::debug!("Successfully deleted space '{id}' from spacedeck");
    }
}

fn build_space_name(room_id: RoomId, room_kind: RoomKind) -> String {
    match room_kind {
        RoomKind::Breakout(breakout_id) => format!("{room_id}-{breakout_id}"),
        RoomKind::Main => format!("{room_id}-main"),
    }
}

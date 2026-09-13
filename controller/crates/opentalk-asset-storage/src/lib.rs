// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    fmt::Display,
    pin::Pin,
    task::{self, Poll},
};

use aws_sdk_s3::primitives::{ByteStream, ByteStreamError};
use bytes::Bytes;
use futures::Stream;
pub use object_storage::{ChunkFormat, ObjectStorage, ObjectStorageError};
use opentalk_inventory::{Asset, Inventory, InventoryProvider, NewAsset, Room};
use opentalk_types_api_internal::module_assets::Quota;
use opentalk_types_common::{
    assets::{AssetFileKind, AssetId, FileExtension},
    events::EventTitle,
    modules::ModuleId,
    rooms::RoomIdOrAlias,
    tariffs::QuotaType,
    time::Timestamp,
    users::UserId,
};
use snafu::{IntoError, ResultExt, Snafu};
pub use storage_notifier::{NoOpStorageNotifier, StorageNotifier};

mod object_storage;
mod storage_notifier;

#[derive(Debug, Snafu)]
pub enum AssetError {
    #[snafu(display("Error connecting to inventory: {source}"))]
    InventoryConnection { source: opentalk_inventory::Error },

    #[snafu(display("Error querying information from inventory: {source}"))]
    InventoryQuery { source: opentalk_inventory::Error },

    #[snafu(display("Failed to upload asset to storage: {source}"))]
    ObjectStorage {
        source: crate::object_storage::ObjectStorageError,
    },

    #[snafu(display("File size too big"))]
    FileSize { source: std::num::TryFromIntError },

    #[snafu(display("The storage quota was exceeded"))]
    AssetStorageExceeded,

    #[snafu(display("The storage quota was exceeded"))]
    // Use AssetError instead of Self, since self will refer to the RollbackSnafu inside the expanded code
    Rollback {
        /// The error that caused the rollback to fail
        #[snafu(source(from(AssetError, Box::new)))]
        source: Box<AssetError>,
        /// The error that required a rollback
        rollback_reason: Box<AssetError>,
    },
}

type Result<T, E = AssetError> = std::result::Result<T, E>;

const ASSET_FILE_NAME_MAX_LENGTH: usize = 100;

#[derive(Debug, Clone)]
pub struct NewAssetFileName {
    event_title: Option<EventTitle>,
    kind: AssetFileKind,
    timestamp: Timestamp,
    extension: FileExtension,
}

impl NewAssetFileName {
    pub fn new(kind: AssetFileKind, timestamp: Timestamp, extension: FileExtension) -> Self {
        Self {
            event_title: None,
            kind,
            timestamp,
            extension,
        }
    }

    pub fn new_with_event_title(
        event_title: Option<EventTitle>,
        kind: AssetFileKind,
        timestamp: Timestamp,
        extension: FileExtension,
    ) -> Self {
        Self {
            event_title,
            kind,
            timestamp,
            extension,
        }
    }
}

impl Display for NewAssetFileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let file_name_fixed_part = format!(
            "{}_{}{}",
            self.kind,
            self.timestamp.to_string_for_filename(),
            self.extension.to_string_with_leading_dot()
        );
        match &self.event_title {
            Some(event_title) if !event_title.is_empty() => {
                let max_length =
                    ASSET_FILE_NAME_MAX_LENGTH.saturating_sub(file_name_fixed_part.len() + 1);
                write!(
                    f,
                    "{}_{}",
                    event_title.sanitized_for_filename(max_length),
                    file_name_fixed_part
                )
            }
            _ => {
                write!(f, "{file_name_fixed_part}")
            }
        }
    }
}

/// The result of a successful asset save operation
#[derive(Debug, Clone)]
pub struct AssetSaved {
    /// The id of the saved asset
    pub asset_id: AssetId,

    /// The filename of the saved asset
    pub filename: String,

    /// The quota after the asset has been saved
    pub quota: Quota,
}

/// Save an asset in the long term storage
///
/// Creates a new database entry before after the asset in the configured S3 bucket.
///
/// If the filename passed in does not have an event title set, this function
/// will attempt to load the title from the event associated with the room if
/// there is any. If no event is associated with the room, the event title will
/// stay empty.
///
/// Returns a tuple containing the asset id and the filename on success.
#[expect(clippy::too_many_arguments)]
pub async fn save_asset<E>(
    storage: &ObjectStorage,
    inventory_provider: &dyn InventoryProvider,
    storage_notifier: &dyn StorageNotifier,
    room: &RoomIdOrAlias,
    namespace: Option<ModuleId>,
    mut filename: NewAssetFileName,
    data: impl Stream<Item = Result<Bytes, E>> + Unpin,
    chunk_format: ChunkFormat,
    max_size: Option<usize>,
) -> Result<AssetSaved>
where
    ObjectStorageError: From<E>,
{
    let mut inventory = inventory_provider
        .get_inventory()
        .await
        .context(InventoryConnectionSnafu)?;
    let (room, storage_quota) = prepare_storage(room, inventory.as_mut()).await?;
    let creator = room.created_by;

    let asset_id = AssetId::generate();

    // Upload to s3 storage
    let size: Result<i64, _> = storage
        .put(&asset_key(&asset_id), data, chunk_format, max_size)
        .await
        .context(ObjectStorageSnafu)
        .and_then(|size| size.try_into().context(FileSizeSnafu));

    let size = match size {
        Ok(size) => size,
        Err(e) => {
            rollback_object_storage(storage, &asset_id).await?;
            return Err(e);
        }
    };

    let result = {
        let mut inventory = inventory_provider
            .get_inventory()
            .await
            .context(InventoryConnectionSnafu)?;
        if filename.event_title.is_none() {
            filename.event_title = inventory
                .get_event_for_room(&room.id.into())
                .await
                .context(InventoryQuerySnafu)?
                .map(|e| e.title);
        }

        let kind = filename.kind.clone();
        let filename = filename.to_string();

        // Create a inventory entry for the uploaded asset
        insert_asset_into_inventory(
            inventory.as_mut(),
            namespace,
            filename.clone(),
            kind,
            asset_id,
            room,
            size,
        )
        .await
        .map(|asset| AssetSaved {
            asset_id: asset.id,
            filename,
            quota: Quota {
                total: storage_quota.total,
                used: storage_quota.used.saturating_add(size as u64),
            },
        })
        .context(InventoryQuerySnafu)
    };

    if let Err(e) = result {
        // if there was an error, we roll back and return the original error.
        // if the rollback fails, we return a rollback error with the cause of
        // the rollback and the reason why the rollback failed.
        return match rollback_object_storage(storage, &asset_id).await {
            Ok(_) => Err(e),
            Err(rollback_err) => Err(rollback_err)
                .with_context(|_| RollbackSnafu::<AssetError> { rollback_reason: e }),
        };
    }

    // Update the room parameters of all roomserver rooms with the same owner
    let new_quota = get_storage_quota(inventory.as_mut(), creator).await?;
    storage_notifier
        .notify(creator, storage_quota, new_quota)
        .await;

    result
}

async fn rollback_object_storage(storage: &ObjectStorage, asset_id: &AssetId) -> Result<()> {
    tracing::info!("Rollback asset upload since room update failed");
    if let Err(rollback_err) = storage.delete(asset_key(asset_id)).await {
        tracing::error!(
            "Failed to rollback s3 asset after database error, leaking asset: {}",
            &asset_key(asset_id)
        );
        Err(ObjectStorageSnafu.into_error(rollback_err))
    } else {
        Ok(())
    }
}

async fn insert_asset_into_inventory(
    inventory: &mut dyn Inventory,
    namespace: Option<ModuleId>,
    filename: String,
    kind: AssetFileKind,
    asset_id: AssetId,
    room: Room,
    size: i64,
) -> opentalk_inventory::Result<Asset> {
    inventory
        .create_asset_for_room(
            room.id,
            NewAsset {
                id: asset_id,
                namespace,
                filename,
                kind: kind.to_string(),
                tenant_id: room.tenant_id,
                size,
            },
        )
        .await
}

async fn prepare_storage(
    room: &RoomIdOrAlias,
    inventory: &mut dyn Inventory,
) -> Result<(Room, Quota), AssetError> {
    let room = inventory
        .get_room(room)
        .await
        .context(InventoryQuerySnafu)?;
    let storage_quota = verify_storage_usage(inventory, room.created_by).await?;
    Ok((room, storage_quota))
}

#[derive(Debug)]
pub struct ByStreamExt(ByteStream);

impl futures::stream::Stream for ByStreamExt {
    type Item = Result<Bytes, ByteStreamError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.0).poll_next(cx)
    }
}

/// Get an asset from the object storage
pub async fn get_asset(
    storage: &ObjectStorage,
    asset_id: &AssetId,
) -> Result<ByStreamExt, crate::object_storage::ObjectStorageError> {
    let stream = storage.get(asset_key(asset_id)).await?;
    Ok(ByStreamExt(stream))
}

/// Delete an asset from the object storage
pub async fn delete_asset(
    storage: &ObjectStorage,
    inventory_provider: &dyn InventoryProvider,
    storage_notifier: &dyn StorageNotifier,
    room: &RoomIdOrAlias,
    asset_id: AssetId,
) -> Result<()> {
    let mut inventory = inventory_provider
        .get_inventory()
        .await
        .context(InventoryConnectionSnafu)?;

    let room = inventory
        .get_room(room)
        .await
        .context(InventoryQuerySnafu)?;
    let creator = room.created_by;
    let old_quota = get_storage_quota(inventory.as_mut(), creator).await?;

    let size: i64 = inventory
        .delete_asset_from_room(room.id, asset_id)
        .await
        .context(InventoryQuerySnafu)?
        .into();

    storage
        .delete(asset_key(&asset_id))
        .await
        .context(ObjectStorageSnafu)?;

    let new_quota = Quota {
        total: old_quota.total,
        used: old_quota.used.saturating_sub(size.unsigned_abs()),
    };
    storage_notifier.notify(creator, old_quota, new_quota).await;

    Ok(())
}

pub fn asset_key(asset_id: &AssetId) -> String {
    format!("assets/{asset_id}")
}

async fn get_storage_quota(inventory: &mut dyn Inventory, user_id: UserId) -> Result<Quota> {
    let total = inventory
        .get_tariff_for_user(user_id)
        .await
        .context(InventoryQuerySnafu)?
        .quota(&QuotaType::MaxStorage);

    let used = inventory
        .get_user_storage_used_size_u64(user_id)
        .await
        .context(InventoryQuerySnafu)?;

    Ok(Quota { total, used })
}

/// Verify that the storage quota wasn't exhausted. Files don't need to fit into the remaining quota,
/// there only needs to be remaining quota.
///
/// # Return Value
///
/// If the storage usage is limited for a user by a storage quota, the current remaining quota is
/// returned. Otherwise `None` is returned.
pub async fn verify_storage_usage(inventory: &mut dyn Inventory, user_id: UserId) -> Result<Quota> {
    let quota = get_storage_quota(inventory, user_id).await?;

    if quota.is_exceeded() {
        AssetStorageExceededSnafu.fail()
    } else {
        Ok(quota)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use chrono::{TimeZone as _, Utc};
    use opentalk_types_common::{
        assets::{FileExtension, asset_file_kind},
        time::Timestamp,
    };
    use pretty_assertions::assert_eq;

    use super::NewAssetFileName;

    #[test]
    fn new_asset_filename() {
        let timestamp = Timestamp::from(Utc.with_ymd_and_hms(2020, 5, 3, 14, 16, 19).unwrap());

        let filename = NewAssetFileName::new(
            asset_file_kind!("recording"),
            timestamp,
            FileExtension::from_str("mkv").unwrap(),
        );
        assert_eq!(
            "recording_2020-05-03_14-16-19-UTC.mkv",
            &filename.to_string()
        );

        let filename = NewAssetFileName::new_with_event_title(
            Some(
                "A very (!!1~) Special Event!"
                    .parse()
                    .expect("valid event title"),
            ),
            asset_file_kind!("meetingnotes_pdf"),
            timestamp,
            FileExtension::pdf(),
        );
        assert_eq!(
            "A very ___1__ Special Event__meetingnotes_pdf_2020-05-03_14-16-19-UTC.pdf",
            &filename.to_string()
        );

        let filename = NewAssetFileName::new_with_event_title(
            Some("世界您好".parse().expect("valid event title")),
            asset_file_kind!("meetingnotes_pdf"),
            timestamp,
            FileExtension::pdf(),
        );
        assert_eq!(
            "世界您好_meetingnotes_pdf_2020-05-03_14-16-19-UTC.pdf",
            &filename.to_string()
        );
    }
}

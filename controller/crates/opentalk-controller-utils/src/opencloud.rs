// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Helpers for working with `OpenCloud` shared folders.

use opentalk_inventory::OpencloudShareReference;
use opentalk_opencloud_client::{DriveId, ItemId, PermissionId};

/// Reference to a created `OpenCloud` share link.
///
/// `OpenCloud` requires the drive, item and permission identifiers to delete a
/// share link again, none of which fit the single share-id model that Nextcloud
/// uses. This is the strongly-typed, client-facing counterpart to the
/// [`OpencloudShareReference`] that is persisted in an event shared folder.
#[derive(Debug, Clone)]
pub struct ShareReference {
    /// Identifier of the drive containing the shared item.
    pub drive_id: DriveId,

    /// Identifier of the shared item.
    pub item_id: ItemId,

    /// Identifier of the permission representing the share link.
    pub permission_id: PermissionId,
}

impl From<OpencloudShareReference> for ShareReference {
    fn from(
        OpencloudShareReference {
            drive_id,
            item_id,
            permission_id,
        }: OpencloudShareReference,
    ) -> Self {
        Self {
            drive_id: DriveId::from(drive_id),
            item_id: ItemId::from(item_id),
            permission_id: PermissionId::from(permission_id),
        }
    }
}

impl From<ShareReference> for OpencloudShareReference {
    fn from(
        ShareReference {
            drive_id,
            item_id,
            permission_id,
        }: ShareReference,
    ) -> Self {
        Self {
            drive_id: drive_id.into(),
            item_id: item_id.into(),
            permission_id: permission_id.into(),
        }
    }
}

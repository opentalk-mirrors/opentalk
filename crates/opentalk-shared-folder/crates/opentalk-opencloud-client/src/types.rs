// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Data transfer objects returned by the OpenCloud LibreGraph API.

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{DriveId, ItemId, PermissionId, SharingLinkType};

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct DriveList {
    pub value: Vec<Drive>,
}

/// A drive (also called a space) the user has access to.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Drive {
    /// The identifier of the drive, e.g. `storage-users-1$<uuid>`.
    pub id: DriveId,
    /// The human-readable name of the drive.
    #[serde(default)]
    pub name: String,
    /// The type of the drive, e.g. `personal` or `project`.
    #[serde(default)]
    pub drive_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct DriveItemList {
    pub value: Vec<DriveItem>,
}

/// An item (file or folder) inside a drive.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveItem {
    /// The identifier of the item.
    pub id: ItemId,
    /// The name of the item.
    #[serde(default)]
    pub name: String,
}

/// A permission as returned by the share-link endpoints.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permission {
    /// The identifier of the permission.
    pub id: PermissionId,
    /// The sharing link associated with this permission, if any.
    #[serde(default)]
    pub link: Option<SharingLink>,
    /// Whether the link is password protected.
    #[serde(default)]
    pub has_password: Option<bool>,
    /// The point in time at which the link expires, if any.
    #[serde(default)]
    pub expiration_date_time: Option<DateTime<Utc>>,
}

/// The sharing link contained in a [`Permission`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharingLink {
    /// The type of the link.
    #[serde(rename = "type")]
    pub link_type: SharingLinkType,
    /// The public URL of the link.
    #[serde(default)]
    pub web_url: String,
}

/// The result of creating a share link, bundling the resolved item id with the
/// created permission so that follow-up updates or deletions need no further
/// path resolution.
#[derive(Debug, Clone)]
pub struct CreatedShareLink {
    /// The identifier of the shared item.
    pub item_id: ItemId,
    /// The permission describing the created link.
    pub permission: Permission,
}

/// The password policy that share-link passwords must conform to, as exposed by
/// the OCS capabilities endpoint.
///
/// When the policy is disabled on the server only `max_characters` is reported;
/// the remaining minimums then default to `0`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PasswordPolicy {
    /// The minimum total number of characters.
    #[serde(default)]
    pub min_characters: u32,
    /// The maximum total number of characters (bytes), usually `72`.
    #[serde(default)]
    pub max_characters: Option<u32>,
    /// The minimum number of lowercase characters.
    #[serde(default)]
    pub min_lowercase_characters: u32,
    /// The minimum number of uppercase characters.
    #[serde(default)]
    pub min_uppercase_characters: u32,
    /// The minimum number of digits.
    #[serde(default)]
    pub min_digits: u32,
    /// The minimum number of special characters.
    #[serde(default)]
    pub min_special_characters: u32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OcsCapabilitiesEnvelope {
    pub ocs: OcsCapabilities,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OcsCapabilities {
    pub data: OcsCapabilitiesData,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OcsCapabilitiesData {
    pub capabilities: Capabilities,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Capabilities {
    #[serde(default)]
    pub password_policy: Option<PasswordPolicy>,
}

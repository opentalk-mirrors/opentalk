// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

/// Provider-specific reference data for an event shared folder.
///
/// The variant determines which backend created the shared folder and carries
/// the identifiers required to manage (and later delete) the created shares. It
/// is stored as internally-tagged JSON in the `provider_data` column, using the
/// `type` field as the discriminator, so the stored payload is self-describing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SharedFolderProvider {
    /// Shared folder backed by a Nextcloud instance.
    Nextcloud {
        /// The share id for the write share.
        write_share_id: String,

        /// The share id for the read share.
        read_share_id: String,
    },

    /// Shared folder backed by an `OpenCloud` instance.
    OpencloudBasic {
        /// Reference to the write share link.
        write: OpencloudShareReference,

        /// Reference to the read share link.
        read: OpencloudShareReference,
    },
}

/// Reference to a created `OpenCloud` share link.
///
/// `OpenCloud` requires the drive, item and permission identifiers to delete a
/// share link again, none of which fit the single share-id model that Nextcloud
/// uses.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpencloudShareReference {
    /// Identifier of the drive containing the shared item.
    pub drive_id: String,

    /// Identifier of the shared item.
    pub item_id: String,

    /// Identifier of the permission representing the share link.
    pub permission_id: String,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn nextcloud_round_trip() {
        let provider = SharedFolderProvider::Nextcloud {
            write_share_id: "write-share".to_string(),
            read_share_id: "read-share".to_string(),
        };
        let json = json!({
            "type": "nextcloud",
            "write_share_id": "write-share",
            "read_share_id": "read-share",
        });

        assert_eq!(serde_json::to_value(&provider).unwrap(), json);
        assert_eq!(
            serde_json::from_value::<SharedFolderProvider>(json).unwrap(),
            provider
        );
    }

    #[test]
    fn opencloud_basic_round_trip() {
        let provider = SharedFolderProvider::OpencloudBasic {
            write: OpencloudShareReference {
                drive_id: "write-drive".to_string(),
                item_id: "write-item".to_string(),
                permission_id: "write-permission".to_string(),
            },
            read: OpencloudShareReference {
                drive_id: "read-drive".to_string(),
                item_id: "read-item".to_string(),
                permission_id: "read-permission".to_string(),
            },
        };
        let json = json!({
            "type": "opencloud_basic",
            "write": {
                "drive_id": "write-drive",
                "item_id": "write-item",
                "permission_id": "write-permission",
            },
            "read": {
                "drive_id": "read-drive",
                "item_id": "read-item",
                "permission_id": "read-permission",
            },
        });

        assert_eq!(serde_json::to_value(&provider).unwrap(), json);
        assert_eq!(
            serde_json::from_value::<SharedFolderProvider>(json).unwrap(),
            provider
        );
    }
}

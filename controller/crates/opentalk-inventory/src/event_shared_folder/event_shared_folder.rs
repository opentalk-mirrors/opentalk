// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    events::EventId,
    shared_folders::{SharedFolder, SharedFolderAccess},
    time::Timestamp,
};

use super::SharedFolderProvider;

/// The representation of an event shared folder in the inventory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventSharedFolder {
    /// The id of the event.
    pub event_id: EventId,

    /// The creation timestamp.
    pub created_at: Timestamp,

    /// The updated timestamp.
    pub updated_at: Timestamp,

    /// The path of the shared folder.
    pub path: String,

    /// The url for the write share.
    pub write_url: String,

    /// The password for the write share.
    pub write_password: String,

    /// The url for the read share.
    pub read_url: String,

    /// The password for the read share.
    pub read_password: String,

    /// The provider-specific reference data for the shared folder.
    pub provider: SharedFolderProvider,
}

impl From<EventSharedFolder> for SharedFolder {
    fn from(
        EventSharedFolder {
            write_password,
            write_url,
            read_password,
            read_url,
            ..
        }: EventSharedFolder,
    ) -> Self {
        SharedFolder {
            read: SharedFolderAccess {
                url: read_url,
                password: read_password,
            },
            read_write: Some(SharedFolderAccess {
                url: write_url,
                password: write_password,
            }),
        }
    }
}

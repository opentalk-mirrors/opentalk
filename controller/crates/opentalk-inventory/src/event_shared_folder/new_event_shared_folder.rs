// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::events::EventId;

use super::SharedFolderProvider;

/// The representation of a new event shared folder that is intended to be stored in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewEventSharedFolder {
    /// The title of the event.
    pub event_id: EventId,

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

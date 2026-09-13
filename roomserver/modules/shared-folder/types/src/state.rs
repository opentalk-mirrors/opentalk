// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Common types related to the shared_folder module

pub use opentalk_types_common::shared_folders::SharedFolderAccess;
use opentalk_types_common::{modules::ModuleId, shared_folders::SharedFolder};
use opentalk_types_signaling::SignalingModuleFrontendData;

use crate::SHARED_FOLDER_MODULE_ID;

/// Information about a shared folder containing
/// read and optional write access
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SharedFolderState {
    /// Read access information for the shared folder
    pub read: SharedFolderAccess,

    /// Read-write access information for the shared folder
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_write: Option<SharedFolderAccess>,
}

impl SharedFolderState {
    /// Get an equivalent shared folder, with write access removed
    pub fn without_write_access(self) -> Self {
        Self {
            read_write: None,
            ..self
        }
    }

    /// Get an equivalent shared folder, with write access added or replaced
    pub fn with_write_access(self, write_access: SharedFolderAccess) -> Self {
        Self {
            read_write: Some(write_access),
            ..self
        }
    }
}

impl From<SharedFolder> for SharedFolderState {
    fn from(value: SharedFolder) -> Self {
        Self {
            read: value.read,
            read_write: value.read_write,
        }
    }
}

impl From<SharedFolderState> for SharedFolder {
    fn from(value: SharedFolderState) -> Self {
        Self {
            read: value.read,
            read_write: value.read_write,
        }
    }
}

impl SignalingModuleFrontendData for SharedFolderState {
    const NAMESPACE: Option<ModuleId> = Some(SHARED_FOLDER_MODULE_ID);
}

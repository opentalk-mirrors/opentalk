// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#![allow(clippy::module_inception)]

mod event_shared_folder;
mod event_shared_folder_inventory;
mod new_event_shared_folder;
mod shared_folder_provider;

pub use event_shared_folder::EventSharedFolder;
pub use event_shared_folder_inventory::EventSharedFolderInventory;
pub use new_event_shared_folder::NewEventSharedFolder;
pub use shared_folder_provider::{OpencloudShareReference, SharedFolderProvider};

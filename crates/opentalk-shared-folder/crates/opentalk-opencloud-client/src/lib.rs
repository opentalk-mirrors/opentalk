// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! OpenCloud client library embedded in OpenTalk.
//!
//! The client is focused on creating folders (via WebDAV) and public share
//! links (via the LibreGraph API) on an OpenCloud instance.

mod client;
mod drive_id;
mod error;
mod item_id;
mod link_creator;
mod link_updater;
mod password;
mod permission_id;
mod sharing_link_type;

pub mod types;

pub use client::Client;
pub use drive_id::DriveId;
pub use error::Error;
pub use item_id::ItemId;
pub use link_creator::CreateLinkOptions;
pub use link_updater::LinkUpdater;
pub use permission_id::PermissionId;
pub use sharing_link_type::SharingLinkType;
pub use types::{CreatedShareLink, Drive, DriveItem, PasswordPolicy, Permission, SharingLink};

type Result<T, E = Error> = std::result::Result<T, E>;

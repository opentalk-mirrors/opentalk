// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! OpenTalk Controller service facade
//!
//! This crate contains traits and data types that provide the service facade
//! which is used by the OpenTalk Controller to provide the Web API.

mod controller_service;
mod middleware;

pub use controller_service::{AssetDownloadProxyStream, OpenTalkControllerService, StartRoomError};
pub use middleware::user::RequestUser;
pub use opentalk_asset_storage::{NewAssetFileName, ObjectStorageError, StorageNotifier};

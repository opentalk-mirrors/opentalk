// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! NextCloud client library embedded in OpenTalk

mod client;
mod error;
mod share_creator;
mod share_id;
mod share_permission;
mod share_type;
mod share_updater;
mod utils;

pub mod types;

pub use client::Client;
pub use error::Error;
pub use share_creator::ShareCreator;
pub use share_id::ShareId;
pub use share_permission::SharePermission;
pub use share_type::ShareType;
pub use share_updater::ShareUpdater;

type Result<T, E = Error> = std::result::Result<T, E>;

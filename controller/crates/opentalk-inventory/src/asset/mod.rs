// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#![allow(clippy::module_inception)]

mod asset;
mod asset_inventory;
mod new_asset;
mod update_asset;

pub use asset::Asset;
pub use asset_inventory::AssetInventory;
pub use new_asset::NewAsset;
pub use update_asset::UpdateAsset;

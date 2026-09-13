// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Data types for handling assets.

mod asset_id;
mod asset_sorting;
mod file_extension;
mod file_size;

pub use asset_id::AssetId;
pub use asset_sorting::AssetSorting;
pub use file_extension::{FILE_EXTENSION_MAX_LENGTH, FileExtension};
pub use file_size::FileSize;
pub use opentalk_types_common_identifiers::asset_file_kind::{
    ASSET_FILE_KIND_MAX_LENGTH, ASSET_FILE_KIND_MIN_LENGTH, AssetFileKind,
};
pub use opentalk_types_common_macros::asset_file_kind;

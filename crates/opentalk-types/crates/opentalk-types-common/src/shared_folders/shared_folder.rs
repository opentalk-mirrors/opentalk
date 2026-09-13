// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Common types related to the shared_folder module

use crate::{shared_folders::SharedFolderAccess, utils::ExampleData};

/// Information about a shared folder containing
/// read and optional write access
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "redis",
    derive(redis_args::ToRedisArgs, redis_args::FromRedisValue)
)]
#[cfg_attr(feature = "redis", to_redis_args(serde), from_redis_value(serde))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(SharedFolder::example_data())),
)]
#[cfg_attr(
    feature = "typescript",
    derive(ts_rs::TS),
    ts(export, export_to = "shared-folder/")
)]
pub struct SharedFolder {
    /// Read access information for the shared folder
    pub read: SharedFolderAccess,

    /// Read-write access information for the shared folder
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub read_write: Option<SharedFolderAccess>,
}

impl ExampleData for SharedFolder {
    fn example_data() -> Self {
        Self {
            read: SharedFolderAccess::example_data(),
            read_write: None,
        }
    }
}

impl SharedFolder {
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

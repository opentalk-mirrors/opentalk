// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::utils::ExampleData;

/// Information required to access a shared folder
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(SharedFolderAccess::example_data())),
)]
#[cfg_attr(
    feature = "typescript",
    derive(ts_rs::TS),
    ts(export_to = "shared-folder/")
)]
pub struct SharedFolderAccess {
    /// Shared folder URL
    pub url: String,

    /// Password required to access the shared folder
    pub password: String,
}

impl ExampleData for SharedFolderAccess {
    fn example_data() -> Self {
        Self {
            url: "https://cloud.example.com/shares/abc123".to_string(),
            password: "v3rys3cr3t".to_string(),
        }
    }
}

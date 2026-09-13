// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Properties by which a list of assets can get sorted.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum AssetSorting {
    /// Sort by filename
    Filename,

    /// Sort by size
    Size,

    /// Sort by namespace
    Namespace,

    /// Sort by kind
    Kind,

    /// Sort by crated at date
    #[default]
    CreatedAt,
}

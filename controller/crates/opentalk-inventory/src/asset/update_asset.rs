// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Representation of an update to an [`super::Asset`] in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateAsset {
    /// Update the size of the asset.
    pub size: Option<i64>,

    /// Update the filename of the asset.
    pub filename: Option<String>,
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::utils::ExampleData;

/// Response for *GET /rooms/{room_id}/assets/{asset_id}/download*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature="utoipa",derive(utoipa::ToSchema), schema(example = json!(AssetDownloadResponseBody::example_data())))]
pub struct AssetDownloadResponseBody {
    /// The download url of the asset
    pub url: String,
}

impl ExampleData for AssetDownloadResponseBody {
    fn example_data() -> Self {
        Self {
            url: "proxy?token=abcdef123456".to_owned(),
        }
    }
}

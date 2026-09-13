// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::utils::ExampleData;

/// Query parameters for *GET /rooms/{room_id}/assets/{asset_id}/download*
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature="utoipa",derive(utoipa::ToSchema), schema(example = json!(AssetDownloadQuery::example_data())))]
pub struct AssetDownloadQuery {
    /// Whether to redirect to the asset URL directly
    /// Defaults to `true`
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub redirect: Option<bool>,
}

impl ExampleData for AssetDownloadQuery {
    fn example_data() -> Self {
        Self {
            redirect: Some(false),
        }
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_api_common::module_assets::AssetResource;
use opentalk_types_common::utils::ExampleData;

/// Response for *POST /rooms/{room_id}/assets*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature="utoipa",derive(utoipa::ToSchema), schema(example = json!(PostAssetResponseBody::example_data())))]
pub struct PostAssetResponseBody(pub AssetResource);

impl ExampleData for PostAssetResponseBody {
    fn example_data() -> Self {
        Self(AssetResource::example_data())
    }
}

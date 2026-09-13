// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::utils::ExampleData;

use super::UserAssetResource;

/// Response body for the `GET /v1/users/me/assets` endpoint
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(GetUserAssetsResponseBody::example_data()))
)]
pub struct GetUserAssetsResponseBody {
    /// Assets owned by the user
    pub owned_assets: Vec<UserAssetResource>,
}

impl ExampleData for GetUserAssetsResponseBody {
    fn example_data() -> Self {
        Self {
            owned_assets: vec![UserAssetResource::example_data()],
        }
    }
}

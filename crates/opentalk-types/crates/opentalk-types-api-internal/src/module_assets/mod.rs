// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! Types related to the asset storage API

mod post_asset_response_body;
mod quota;

pub use opentalk_types_api_common::module_assets::AssetResource;
pub use post_asset_response_body::PostAssetResponseBody;
pub use quota::Quota;

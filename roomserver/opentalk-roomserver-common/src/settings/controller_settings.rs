// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_service_auth::ApiKey;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct ControllerConfig {
    // The controller url
    pub url: Url,

    // The API key for the controllers services
    pub api_key: ApiKey,
}

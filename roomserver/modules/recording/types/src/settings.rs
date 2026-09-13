// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types::module_settings::SignalingModuleSettings;
use opentalk_service_auth::ApiKey;
use opentalk_types_common::{modules::ModuleId, utils::ExampleData};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::RECORDING_MODULE_ID;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordingSettings {
    /// The recorder service base url.
    pub url: Url,

    /// The API key for signing recorder JWT tokens.
    pub api_key: ApiKey,
}

impl SignalingModuleSettings for RecordingSettings {
    const NAMESPACE: ModuleId = RECORDING_MODULE_ID;
}

impl ExampleData for RecordingSettings {
    fn example_data() -> Self {
        Self {
            url: Url::parse("http://localhost:5555").expect("Example URL must be valid"),
            api_key: ApiKey::new("recorder", "secret"),
        }
    }
}

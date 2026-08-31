// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types::module_settings::SignalingModuleSettings;
use opentalk_types_common::{modules::ModuleId, utils::ExampleData};
use url::Url;

use crate::LIVEKIT_MODULE_ID;

/// LiveKit settings.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LiveKitSettings {
    /// The API key for connecting to LiveKit.
    pub api_key: String,

    /// The API secret for connecting to LiveKit.
    pub api_secret: String,

    /// The public url that OpenTalk clients will use for connecting to LiveKit.
    pub public_url: String,

    /// The url that the OpenTalk controller will use for connecting to LiveKit.
    pub service_url: Url,
}

impl SignalingModuleSettings for LiveKitSettings {
    const NAMESPACE: ModuleId = LIVEKIT_MODULE_ID;
}

impl ExampleData for LiveKitSettings {
    fn example_data() -> Self {
        Self {
            api_key: "devkey".to_owned(),
            api_secret: "secret".to_owned(),
            public_url: "http://localhost:7880".to_owned(),
            service_url: Url::parse("http://localhost:7880").expect("Example URL must be valid"),
        }
    }
}

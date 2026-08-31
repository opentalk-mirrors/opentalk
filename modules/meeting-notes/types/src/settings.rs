// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_types::module_settings::SignalingModuleSettings;
use opentalk_types_common::{modules::ModuleId, utils::ExampleData};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::MEETING_NOTES_MODULE_ID;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeetingNotesSettings {
    /// The base URL of the Etherpad instance.
    pub base_url: Url,

    /// The API key for accessing the Etherpad instance.
    pub api_key: String,
}

impl SignalingModuleSettings for MeetingNotesSettings {
    const NAMESPACE: ModuleId = MEETING_NOTES_MODULE_ID;
}

impl ExampleData for MeetingNotesSettings {
    fn example_data() -> Self {
        Self {
            base_url: Url::parse("http://localhost:9001").expect("Example URL must be valid"),
            api_key: "secret".to_owned(),
        }
    }
}

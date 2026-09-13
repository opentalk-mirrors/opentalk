// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// The current credentials of the livekit instance
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Credentials {
    /// The room id
    pub room: String,
    /// The token for the service / frontend
    pub token: String,
    /// The "public" livekit URL
    pub public_url: String,
    /// The livekit URL to be used by services
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_url: Option<String>,
}

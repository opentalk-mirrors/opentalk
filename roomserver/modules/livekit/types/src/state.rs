// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{Credentials, MicrophoneRestrictionState};

/// Signaling event to pass information about the livekit server around
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LiveKitState {
    /// The current credentials for accessing a room on the livekit instance
    #[serde(flatten)]
    pub credentials: Credentials,

    /// The current state of microphone restrictions
    pub microphone_restriction_state: MicrophoneRestrictionState,
}

impl opentalk_types_signaling::SignalingModuleFrontendData for LiveKitState {
    const NAMESPACE: Option<opentalk_types_common::modules::ModuleId> =
        Some(crate::LIVEKIT_MODULE_ID);
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

/// A flag to track the participants ready status
#[derive(Default, Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TimerPeerState {
    /// The ready status of the participant
    pub ready_status: bool,
}

impl opentalk_types_signaling::SignalingModulePeerFrontendData for TimerPeerState {
    const NAMESPACE: Option<opentalk_types_common::modules::ModuleId> =
        Some(crate::TIMER_MODULE_ID);
}

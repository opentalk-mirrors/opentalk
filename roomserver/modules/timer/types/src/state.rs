// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use crate::TimerConfig;

/// Status of and belonging to a currently active timer
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimerState {
    /// config of the timer
    #[serde(flatten)]
    pub config: TimerConfig,

    /// Flag to indicate that the current participant has marked themselves as ready
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ready_status: Option<bool>,
}

impl opentalk_types_signaling::SignalingModuleFrontendData for TimerState {
    const NAMESPACE: Option<opentalk_types_common::modules::ModuleId> =
        Some(crate::TIMER_MODULE_ID);
}

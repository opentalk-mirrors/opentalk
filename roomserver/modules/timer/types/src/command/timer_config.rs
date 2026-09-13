// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::time::Timestamp;
use serde::{Deserialize, Serialize};

use crate::Kind;

/// Status of a currently active timer
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimerConfig {
    /// start time of the timer
    pub started_at: Timestamp,
    /// Timer kind
    #[serde(flatten)]
    pub kind: Kind,
    /// Style to use for the timer. Set by the sender.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// The optional title of the timer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Flag to allow/disallow participants to mark themselves as ready
    pub ready_check_enabled: bool,
}

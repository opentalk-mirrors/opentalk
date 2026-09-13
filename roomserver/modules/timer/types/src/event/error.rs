// SPDX-License-Identifier: EUPL-1.2
//
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_types::signaling::module_error::ModuleError;

/// Errors from the `timer` module namespace
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "error")]
pub enum TimerError {
    /// An invalid timer duration has been configured
    InvalidDuration,
    /// The requesting user has insufficient permissions
    InsufficientPermissions,
    /// A timer is already running
    TimerAlreadyRunning,
    /// An internal error occurred and the timer was stopped
    Internal,
    /// The timer is not running
    TimerNotRunning,
    /// The timer ready check is not enabled
    ReadyCheckNotEnabled,
}

impl ModuleError for TimerError {}

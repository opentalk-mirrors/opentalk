// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_signaling::storage::assets::StorageError;
use opentalk_roomserver_types::signaling::module_error::ModuleError;
use serde::{Deserialize, Serialize};

/// Error from the `meeting_report` module namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum TrainingParticipationReportError {
    /// The requesting user has insufficient permissions for the operation
    InsufficientPermissions,

    /// The creator attempted to enable presence logging when it was already enabled.
    PresenceLoggingAlreadyEnabled,

    /// A frontend attempted to perform an action that requires enabled presence logging when it
    /// wasn't enabled.
    PresenceLoggingNotEnabled,

    /// A participant who shouldn't confirm the presence attempted to do so.
    PresenceLoggingNotAllowedForParticipant,

    /// Storage exceeded
    StorageExceeded,

    /// Internal error while generating the report
    Generate,

    /// An internal error occurred
    Internal,

    /// Internal error while saving the report
    Storage,
}

impl ModuleError for TrainingParticipationReportError {}

impl From<StorageError> for TrainingParticipationReportError {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::QuotaExceeded => Self::StorageExceeded,
            StorageError::Internal(..) | StorageError::ReadAsset(..) => Self::Storage,
        }
    }
}

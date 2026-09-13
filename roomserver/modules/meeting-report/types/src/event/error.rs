// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_signaling::storage::assets::StorageError;
use opentalk_roomserver_types::signaling::module_error::ModuleError;
use serde::{Deserialize, Serialize};

/// Error from the `meeting_report` module namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum MeetingReportError {
    /// The requesting user has insufficient permissions for the operation
    InsufficientPermissions,
    /// The requesting user has exceeded their storage
    StorageExceeded,
    /// Internal error while generating the report
    Generate,
    /// Internal error while saving the report
    Storage,
}

impl From<StorageError> for MeetingReportError {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::QuotaExceeded => MeetingReportError::StorageExceeded,
            StorageError::Internal(..) | StorageError::ReadAsset(..) => MeetingReportError::Storage,
        }
    }
}

impl ModuleError for MeetingReportError {}

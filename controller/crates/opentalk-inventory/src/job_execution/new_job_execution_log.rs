// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::time::Timestamp;

use super::{JobExecutionId, JobExecutionLogLevel};

/// The representation of a new job execution log that is intended to be stored in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewJobExecutionLog {
    /// The id of the job execution.
    pub execution_id: JobExecutionId,

    /// The logging timestamp.
    pub logged_at: Timestamp,

    /// The log level.
    pub log_level: JobExecutionLogLevel,

    /// The log message.
    pub log_message: String,
}

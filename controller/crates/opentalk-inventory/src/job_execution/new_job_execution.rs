// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::time::Timestamp;

use super::{JobId, JobStatus};

/// The representation of a new job execution that is intended to be stored in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewJobExecution {
    /// The id of the job.
    pub job_id: JobId,

    /// The timestamp when the job execution was started.
    pub started_at: Timestamp,

    /// An optional timestamp when the job execution ended.
    pub ended_at: Option<Timestamp>,

    /// The status of the job execution.
    pub job_status: JobStatus,
}

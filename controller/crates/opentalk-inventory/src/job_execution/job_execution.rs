// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::time::Timestamp;

use super::{JobExecutionId, JobId, JobStatus};

/// The representation of a maintenace job execution in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobExecution {
    /// The id of the job execution.
    pub id: JobExecutionId,

    /// The id of the job.
    pub job_id: JobId,

    /// The timestamp when the job execution was started.
    pub started_at: Timestamp,

    /// An optional timestamp when the job execution ended.
    pub ended_at: Option<Timestamp>,

    /// The status of the job execution.
    pub job_status: JobStatus,
}

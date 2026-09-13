// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::time::Timestamp;

use super::JobStatus;

/// Representation of an update to a job execution in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateJobExecution {
    /// An optional timestamp when the job execution ended.
    pub ended_at: Option<Timestamp>,

    /// The status of the job execution.
    pub job_status: Option<JobStatus>,
}

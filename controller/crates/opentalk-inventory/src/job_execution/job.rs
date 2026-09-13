// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::{JobId, JobType};

/// The representation of a maintenace job in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    /// The id of the job.
    pub id: JobId,

    /// The name of the job.
    pub name: String,

    /// The kind of job.
    pub kind: JobType,

    /// The parameters passed to the job when it is executed.
    pub parameters: serde_json::Value,

    /// The timeout duration.
    pub timeout_secs: i32,

    /// The recurrence pattern for the job.
    pub recurrence: String,
}

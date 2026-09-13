// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains job executions table structs

mod job_execution;
mod job_status;
mod new_job_execution;
mod update_job_execution;

pub use job_execution::JobExecution;
pub use job_status::{JobStatus, JobStatusType};
pub use new_job_execution::NewJobExecution;
pub use update_job_execution::UpdateJobExecution;

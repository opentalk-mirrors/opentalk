// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::{
    Job, JobExecution, JobExecutionId, JobId, NewJobExecution, NewJobExecutionLog,
    UpdateJobExecution,
};
use crate::Result;

/// A trait for retrieving and storing job execution log entities.
#[async_trait::async_trait]
pub trait JobExecutionInventory {
    /// Get a job by its id.
    async fn get_job(&mut self, job_id: JobId) -> Result<Job>;

    /// Get all jobs.
    async fn get_all_jobs(&mut self) -> Result<Vec<Job>>;

    /// Create a new job execution.
    async fn create_job_execution(
        &mut self,
        job_execution: NewJobExecution,
    ) -> Result<JobExecution>;

    /// Update a job execution.
    async fn update_job_execution(
        &mut self,
        job_execution_id: JobExecutionId,
        job_execution: UpdateJobExecution,
    ) -> Result<JobExecution>;

    /// Create a new batch of job execution logs.
    async fn create_job_execution_logs(
        &mut self,
        job_execution_logs: &[NewJobExecutionLog],
    ) -> Result<()>;
}

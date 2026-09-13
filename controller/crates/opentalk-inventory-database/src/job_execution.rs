// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage as db;
use opentalk_inventory::{
    Job, JobExecution, JobExecutionId, JobExecutionInventory, JobId, NewJobExecution,
    NewJobExecutionLog, UpdateJobExecution,
};
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result, error::DatabaseSnafu};

#[async_trait::async_trait]
impl JobExecutionInventory for DatabaseConnection {
    async fn get_job(&mut self, job_id: JobId) -> Result<Job> {
        Ok(db::queries::jobs::get_job(&mut self.inner, job_id.into())
            .await
            .context(DatabaseSnafu)?
            .into())
    }

    async fn get_all_jobs(&mut self) -> Result<Vec<Job>> {
        Ok(db::queries::jobs::get_all_jobs(&mut self.inner)
            .await
            .context(DatabaseSnafu)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    async fn update_job_execution(
        &mut self,
        job_execution_id: JobExecutionId,
        job_execution: UpdateJobExecution,
    ) -> Result<JobExecution> {
        Ok(db::queries::jobs::update_job_execution(
            &mut self.inner,
            job_execution.into(),
            job_execution_id.into(),
        )
        .await
        .context(DatabaseSnafu)?
        .into())
    }

    async fn create_job_execution(
        &mut self,
        job_execution: NewJobExecution,
    ) -> Result<JobExecution> {
        Ok(
            db::queries::jobs::create_job_execution(&mut self.inner, job_execution.into())
                .await
                .context(DatabaseSnafu)?
                .into(),
        )
    }

    /// Create a new batch of job execution logs.
    async fn create_job_execution_logs(
        &mut self,
        job_execution_logs: &[NewJobExecutionLog],
    ) -> Result<()> {
        // TODO: this has much worse performance than it should,
        // we should get rid of the clone and perform the insertion directly.
        let job_execution_logs: Vec<_> = job_execution_logs
            .iter()
            .cloned()
            .map(db::tables::job_execution_logs::NewJobExecutionLog::from)
            .collect();
        Ok(
            db::queries::jobs::create_job_execution_logs(&mut self.inner, &job_execution_logs)
                .await
                .context(DatabaseSnafu)?,
        )
    }
}

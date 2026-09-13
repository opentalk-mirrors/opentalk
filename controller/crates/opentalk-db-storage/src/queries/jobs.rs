// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains jobs database queries

use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use opentalk_database::{DatabaseError, DbConnection, Result};

use crate::{
    schema::{job_execution_logs, job_executions, jobs},
    tables::{
        job_execution_logs::NewJobExecutionLog,
        job_executions::{JobExecution, NewJobExecution, UpdateJobExecution},
        jobs::{Job, SerialJobId},
    },
};

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_job(conn: &mut DbConnection, id: SerialJobId) -> Result<Job> {
    jobs::table
        .filter(jobs::id.eq(id))
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_all_jobs(conn: &mut DbConnection) -> Result<Vec<Job>> {
    jobs::table.load(conn).await.map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn create_job_execution(
    conn: &mut DbConnection,
    new_job_execution: NewJobExecution,
) -> Result<JobExecution> {
    diesel::insert_into(job_executions::table)
        .values(new_job_execution)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn update_job_execution(
    conn: &mut DbConnection,
    update_job_execution: UpdateJobExecution,
    id: SerialJobId,
) -> Result<JobExecution> {
    let target = job_executions::table.filter(job_executions::id.eq(&id));

    diesel::update(target)
        .set(update_job_execution)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn create_job_execution_logs(
    conn: &mut DbConnection,
    batch: &[NewJobExecutionLog],
) -> Result<()> {
    // todo: is there a maximum amount of rows for batch inserts?
    diesel::insert_into(job_execution_logs::table)
        .values(batch)
        .execute(conn)
        .await?;

    Ok(())
}

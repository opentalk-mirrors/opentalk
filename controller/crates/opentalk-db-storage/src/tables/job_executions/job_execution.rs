// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use diesel::{Identifiable, Queryable};
use opentalk_inventory as inventory;

use crate::{
    schema::job_executions,
    tables::{job_executions::JobStatus, jobs::SerialJobId},
};

#[derive(Debug, Clone, Queryable, Identifiable, PartialEq, Eq)]
#[diesel(table_name = job_executions)]
pub struct JobExecution {
    pub id: SerialJobId,
    pub job_id: SerialJobId,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub job_status: JobStatus,
}

impl From<JobExecution> for inventory::JobExecution {
    fn from(
        JobExecution {
            id,
            job_id,
            started_at,
            ended_at,
            job_status,
        }: JobExecution,
    ) -> Self {
        Self {
            id: id.into(),
            job_id: job_id.into(),
            started_at: started_at.into(),
            ended_at: ended_at.map(Into::into),
            job_status: job_status.into(),
        }
    }
}

impl From<inventory::JobExecution> for JobExecution {
    fn from(
        inventory::JobExecution {
            id,
            job_id,
            started_at,
            ended_at,
            job_status,
        }: inventory::JobExecution,
    ) -> Self {
        Self {
            id: id.into(),
            job_id: job_id.into(),
            started_at: started_at.into(),
            ended_at: ended_at.map(Into::into),
            job_status: job_status.into(),
        }
    }
}

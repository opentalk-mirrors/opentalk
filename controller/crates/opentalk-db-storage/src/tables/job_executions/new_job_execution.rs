// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use diesel::Insertable;
use opentalk_inventory as inventory;

use crate::{
    schema::job_executions,
    tables::{job_executions::JobStatus, jobs::SerialJobId},
};

#[derive(Debug, Insertable)]
#[diesel(table_name = job_executions)]
pub struct NewJobExecution {
    pub job_id: SerialJobId,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub job_status: JobStatus,
}

impl From<inventory::NewJobExecution> for NewJobExecution {
    fn from(
        inventory::NewJobExecution {
            job_id,
            started_at,
            ended_at,
            job_status,
        }: inventory::NewJobExecution,
    ) -> Self {
        Self {
            job_id: job_id.into(),
            started_at: started_at.into(),
            ended_at: ended_at.map(Into::into),
            job_status: job_status.into(),
        }
    }
}

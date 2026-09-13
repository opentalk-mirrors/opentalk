// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use diesel::{Identifiable, Queryable};

use crate::{
    schema::job_execution_logs,
    tables::{job_execution_logs::LogLevel, jobs::SerialJobId},
};

#[derive(Debug, Clone, Queryable, Identifiable, PartialEq, Eq)]
#[diesel(table_name = job_execution_logs)]
pub struct JobExecutionLog {
    pub id: SerialJobId,
    pub execution_id: SerialJobId,
    pub logged_at: DateTime<Utc>,
    pub log_level: LogLevel,
    pub log_message: String,
}

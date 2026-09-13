// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains job execution logs table structs

mod job_execution_log;
mod log_level;
mod new_job_execution_log;

pub use job_execution_log::JobExecutionLog;
pub use log_level::{LogLevel, LogLevelType};
pub use new_job_execution_log::NewJobExecutionLog;

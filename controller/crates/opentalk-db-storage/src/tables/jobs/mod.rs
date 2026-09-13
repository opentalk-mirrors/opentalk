// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains jobs table structs

mod job;
mod job_type;
mod serial_job_id;

pub use job::Job;
pub use job_type::{JobType, JobTypeType};
pub use serial_job_id::SerialJobId;

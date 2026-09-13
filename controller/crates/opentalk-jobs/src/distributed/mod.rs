// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains the components for a system that allows one of many controllers to be elected as a job executor. There can
//! only be one job executor at a time. In case of a failure, another running controller will take over.
//!
//! An etcd setup is required for the leader election, this system only starts when the `[etcd]` config values are set.
//!
//! The elected job executor reads the configured jobs and their parameters from the database and puts them in a queue
//! based on their schedule. These jobs are then run sequentially from the queue, their progress and logs are saved in
//! the database.
//!
//! The [`JobRunner`](job_runner::JobRunner) is the main entry point to start and manage all related tasks:
//!
//! [`ElectionTask`](election_task::ElectionTask) - Manages the etcd leader election
//! [`JobQueue`](job_queue::JobQueue) - A cron based scheduler that create the job queue entries in etcd
//! [`JobExecutor`](executor_task::JobExecutor) - The task that watches the queue and executes jobs
//! [`ExecutionLogger`](execution_logger::ExecutionLogger) - A logger that writes to the database
//!

use opentalk_inventory::JobId;
mod election_task;
mod execution_logger;
mod executor_task;
mod job_queue;
pub mod job_runner;

/// The general etcd lease time to live in seconds
pub const ETCD_LEASE_TTL: u64 = 6;

/// The etcd prefix key for the distributed job queue
pub const JOB_QUEUE_PREFIX: &str = "opentalk/jobs/queue/";
/// The etcd prefix key to mark jobs as running
pub const JOB_RUNNING_PREFIX: &str = "opentalk/jobs/running/";

pub fn build_queue_key(job_id: JobId) -> String {
    format!("{JOB_QUEUE_PREFIX}job_{job_id}")
}

pub fn build_running_key(job_id: JobId) -> String {
    format!("{JOB_RUNNING_PREFIX}job_{job_id}")
}

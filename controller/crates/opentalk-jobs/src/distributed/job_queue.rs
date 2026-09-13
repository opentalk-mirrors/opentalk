// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{HashMap, HashSet};

use etcd_client::{Client, Compare, CompareOp, TxnOp};
use opentalk_inventory::{Job, JobId};
use snafu::{ResultExt, Snafu};
use tokio_cron_scheduler::{JobScheduler, JobSchedulerError};
use uuid::Uuid;

use super::build_queue_key;
use crate::error::{AddSnafu, ConnectSnafu};

#[derive(Debug, Snafu)]
pub enum QueueError {
    #[snafu(display("Failed to add job '{job_id}'to scheduler"))]
    QueueAdd {
        job_id: i64,
        source: JobSchedulerError,
    },

    #[snafu(display("Failed to remove job '{job_id}' from scheduler"))]
    QueueRemove {
        job_id: i64,
        source: JobSchedulerError,
    },

    #[snafu(display("Failed to create cron job task for job with id {job_id}"))]
    CreateJob {
        job_id: i64,
        source: JobSchedulerError,
    },

    #[snafu(display("Failed to create cron job scheduler"))]
    CreateScheduler { source: JobSchedulerError },

    #[snafu(display("Failed to initialize cron job scheduler"))]
    InitScheduler { source: JobSchedulerError },

    #[snafu(display("Failed to start cron job scheduler"))]
    StartScheduler { source: JobSchedulerError },

    #[snafu(display("Failed to shutdown scheduler"))]
    ShutdownScheduler { source: JobSchedulerError },

    /// An etcd API error occurred
    #[snafu(transparent)]
    Etcd { source: crate::error::EtcdError },
}

#[derive(Debug)]
pub(crate) struct JobInfo {
    /// The id that was assigned in the internal scheduler
    pub cron_id: Uuid,
    /// The job and its configuration
    pub job: Job,
}

/// A wrapper for a [`tokio_cron_scheduler::JobScheduler`]
///
/// Maintains a list of jobs and puts them in a queue when their specified cron schedule occurs.
pub struct JobQueue {
    /// The etcd client
    client: Client,
    /// The internal cron scheduler
    scheduler: JobScheduler,
    /// A map of job ids and their related
    job_table: HashMap<JobId, JobInfo>,
}

impl JobQueue {
    /// Connect to etcd and initialize the cron scheduler.
    pub(crate) async fn init(etcd_urls: Vec<String>) -> Result<Self, QueueError> {
        let client = Client::connect(&etcd_urls, None)
            .await
            .context(ConnectSnafu)?;

        let mut scheduler = JobScheduler::new().await.context(CreateSchedulerSnafu)?;

        scheduler.init().await.context(InitSchedulerSnafu)?;

        Ok(Self {
            client,
            scheduler,
            job_table: HashMap::new(),
        })
    }

    pub(crate) fn current_jobs(&self) -> HashSet<JobId> {
        self.job_table.keys().copied().collect()
    }

    /// Add a new job or update it if it already exists
    pub(crate) async fn add_or_update(&mut self, job: Job) -> Result<(), QueueError> {
        if let Some(job_info) = self.job_table.get(&job.id) {
            if job_info.job == job {
                // The same job already exists
                return Ok(());
            }

            if let Some(job_info) = self.remove(job.id).await? {
                log::debug!(
                    "Removed job in preparation for update (id: {}, name: {}, kind: {}, cron schedule: {}",
                    job_info.job.id,
                    job_info.job.name,
                    job_info.job.kind,
                    job_info.job.recurrence
                );
            }
        }

        if let Some(job_info) = self.add(job).await? {
            log::debug!(
                "Added job (id: {}, name: {}, kind: {}, cron schedule: {}",
                job_info.job.id,
                job_info.job.name,
                job_info.job.kind,
                job_info.job.recurrence
            );
        }

        Ok(())
    }

    /// Adds a new job to the scheduler
    ///
    /// Returns `Ok(None)` if a job with the same job id already exists.
    pub(crate) async fn add(&mut self, job: Job) -> Result<Option<JobInfo>, QueueError> {
        let job_id = job.id;

        if self.job_table.contains_key(&job_id) {
            log::debug!("Job {job_id} already exists in scheduler");
            return Ok(None);
        }

        let client = self.client.clone();

        let cron_job =
            tokio_cron_scheduler::Job::new_async(job.recurrence.as_str(), move |_uuid, _l| {
                let client = client.clone();

                Box::pin(async move {
                    log::debug!("adding job to queue {job_id}");
                    if let Err(e) = add_job_to_queue(job_id, client).await {
                        log::error!("Failed to add job `{e}` to job queue, discarding job:");
                    }
                })
            })
            .context(CreateJobSnafu { job_id })?;

        let uuid = self
            .scheduler
            .add(cron_job)
            .await
            .context(QueueAddSnafu { job_id })?;

        let job_info = JobInfo {
            cron_id: uuid,
            job: job.clone(),
        };

        let job = self.job_table.insert(job.id, job_info);

        Ok(job)
    }

    /// Removes a Job from the scheduler
    pub(crate) async fn remove(&mut self, job_id: JobId) -> Result<Option<JobInfo>, QueueError> {
        match self.job_table.remove(&job_id) {
            Some(job_info) => {
                self.scheduler
                    .remove(&job_info.cron_id)
                    .await
                    .context(QueueRemoveSnafu { job_id })?;

                Ok(Some(job_info))
            }
            None => Ok(None),
        }
    }

    /// Starts the internal cron scheduler task
    pub(crate) async fn start(&mut self) -> Result<(), QueueError> {
        let mut scheduler = JobScheduler::new().await.context(CreateSchedulerSnafu)?;
        scheduler.init().await.context(InitSchedulerSnafu)?;
        scheduler.start().await.context(StartSchedulerSnafu)?;

        self.scheduler = scheduler;

        Ok(())
    }

    /// Stops the internal cron scheduler task
    pub(crate) async fn stop(&mut self) -> Result<(), QueueError> {
        for job_id in self.current_jobs() {
            self.remove(job_id).await?;
        }

        self.scheduler
            .shutdown()
            .await
            .context(ShutdownSchedulerSnafu)?;

        Ok(())
    }
}

/// Creates a new job queue entry in etcd
///
/// The creation fails when the specific job already exists in the queue
///
/// Returns false when the key already exists
async fn add_job_to_queue(job_id: JobId, mut client: Client) -> Result<bool, QueueError> {
    let key = build_queue_key(job_id);

    let txn = etcd_client::Txn::new()
        .when([Compare::version(key.as_bytes(), CompareOp::Equal, 0)])
        .and_then([TxnOp::put(key.as_bytes(), job_id.to_string(), None)]);

    let response = client.txn(txn).await.context(AddSnafu { key })?;

    Ok(response.succeeded())
}

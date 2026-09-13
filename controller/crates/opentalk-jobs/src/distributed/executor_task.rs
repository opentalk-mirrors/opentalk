// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{borrow::BorrowMut, sync::Arc, time::Duration};

use etcd_client::{
    Client, Compare, CompareOp, EventType, GetOptions, KeyValue, PutOptions, TxnOp, WatchOptions,
};
use log::Log;
use opentalk_controller_api_authorization::authorization::Authorizer;
use opentalk_controller_settings::Settings;
use opentalk_controller_utils::deletion::StopRoomBackend;
use opentalk_inventory::{
    InventoryProvider, JobId, JobStatus, JobType, NewJobExecution, UpdateJobExecution,
};
use opentalk_types_common::time::Timestamp;
use snafu::{ResultExt, Snafu};
use tokio::{sync::oneshot, task::JoinHandle, time::interval};

use super::{
    ETCD_LEASE_TTL, JOB_QUEUE_PREFIX, JOB_RUNNING_PREFIX, build_queue_key, build_running_key,
    execution_logger::{ExecutionLogger, ExecutionLoggerError},
};
use crate::{
    Job as JobImpl,
    error::{
        ConnectSnafu, CreateKeepAliveSnafu, CreateWatchSnafu, GetSnafu, KeepAliveSnafu, LeaseSnafu,
        ParseSnafu, RemoveSnafu, WatchProgressSnafu,
    },
    jobs::{
        AdhocEventCleanup, EventCleanup, InviteCleanup, KeycloakAccountSync, RoomCleanup,
        SelfCheck, SyncStorageFiles, UserCleanup,
    },
};

#[derive(Debug, Snafu)]
pub enum ExecutorError {
    #[snafu(transparent)]
    ExecutionLoggerError { source: ExecutionLoggerError },

    #[snafu(display("{msg}: {source}"))]
    Inventory {
        msg: String,
        source: opentalk_inventory::Error,
    },

    #[snafu(transparent)]
    EtcdError {
        #[snafu(source(from(crate::error::EtcdError, Box::new)))]
        source: Box<crate::error::EtcdError>,
    },

    #[snafu(display("The watch stream threw an error: {source}"))]
    WatchStream {
        #[snafu(source(from(etcd_client::Error, Box::new)))]
        source: Box<etcd_client::Error>,
    },

    #[snafu(display("The watch stream closed unexpectedly"))]
    WatchStreamClosed,

    #[snafu(display("Failed to cleanup job keys: {source}"))]
    ClearJobError {
        #[snafu(source(from(etcd_client::Error, Box::new)))]
        source: Box<etcd_client::Error>,
    },

    #[snafu(display("Failed to mark job as running: {source}"))]
    MarkRunning {
        #[snafu(source(from(etcd_client::Error, Box::new)))]
        source: Box<etcd_client::Error>,
    },

    #[snafu(display("Failed to parse job id from {val}: {source}"))]
    ParseIntError {
        val: String,
        source: std::num::ParseIntError,
    },

    #[snafu(display("Executor task panicked: {source}"))]
    ExecutorTaskPanicked { source: tokio::task::JoinError },

    #[snafu(display("Keep alive task panicked: {source}"))]
    KeepAliveTaskPanicked { source: tokio::task::JoinError },
}

/// A handle to interact and monitor the executor task
pub struct JobExecutorHandle {
    etcd_urls: Vec<String>,
    settings: Arc<Settings>,
    inventory_provider: Arc<dyn InventoryProvider>,
    authorizer: Authorizer,
    stop_room_backend: Arc<dyn StopRoomBackend>,
    /// Handle to the inner JobExecutor task
    inner_handle: Option<InnerHandle>,
}

struct InnerHandle {
    /// Used to notify the receiver when the handle is dropped
    shutdown: oneshot::Sender<()>,
    /// JoinHandle to check if the task has exited or
    join_handle: JoinHandle<Result<(), ExecutorError>>,
}

impl JobExecutorHandle {
    pub async fn new(
        etcd_urls: Vec<String>,
        inventory_provider: Arc<dyn InventoryProvider>,
        authorizer: Authorizer,
        stop_room_backend: Arc<dyn StopRoomBackend>,
        settings: Arc<Settings>,
    ) -> Self {
        Self {
            etcd_urls,
            settings,
            inventory_provider,
            authorizer,
            stop_room_backend,
            inner_handle: None,
        }
    }

    /// Start the job executor task
    pub async fn start(&mut self) -> Result<(), ExecutorError> {
        if self.inner_handle.is_some() {
            //already running
            return Ok(());
        }

        log::debug!("starting job executor");

        let handle = JobExecutor::start(
            self.etcd_urls.clone(),
            Arc::clone(&self.inventory_provider),
            self.authorizer.clone(),
            Arc::clone(&self.stop_room_backend),
            self.settings.clone(),
        )
        .await?;

        self.inner_handle = Some(handle);

        Ok(())
    }

    /// Stops the executor task
    ///
    /// Does nothing when the executor task is not running.
    ///
    /// The executor will finish any running job before exiting
    pub fn stop(&mut self) {
        if let Some(handle) = self.inner_handle.take() {
            log::debug!("stopping job executor");
            let _ = handle.shutdown.send(());
        }
    }

    /// Awaits the inner join handle to check if the task errored
    ///
    /// Returns `Ok(())` when when the task finished or no task exists
    pub async fn join(&mut self) -> Result<(), ExecutorError> {
        if let Some(inner_handle) = &mut self.inner_handle {
            let result = inner_handle.join_handle.borrow_mut().await;

            self.inner_handle = None;

            return result.context(ExecutorTaskPanickedSnafu)?;
        }

        Ok(())
    }
}

/// Executes jobs from the etcd job queue
///
/// Jobs are run in sequence to avoid potential collisions between multiple jobs.
pub(crate) struct JobExecutor {
    settings: Arc<Settings>,
    inventory_provider: Arc<dyn InventoryProvider>,
    authorizer: Authorizer,
    stop_room_backend: Arc<dyn StopRoomBackend>,
    client: Client,
    lease_id: i64,
    keep_alive_handle: JoinHandle<Result<(), ExecutorError>>,
    shutdown: oneshot::Receiver<()>,
}

impl JobExecutor {
    async fn start(
        etcd_urls: Vec<String>,
        inventory_provider: Arc<dyn InventoryProvider>,
        authorizer: Authorizer,
        stop_room_backend: Arc<dyn StopRoomBackend>,
        settings: Arc<Settings>,
    ) -> Result<InnerHandle, ExecutorError> {
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let mut client = Client::connect(etcd_urls, None)
            .await
            .context(ConnectSnafu)?;

        let (lease_id, keep_alive_handle) = Self::spawn_keep_alive_task(&mut client).await?;

        let this = Self {
            settings,
            inventory_provider,
            authorizer,
            stop_room_backend,
            client,
            lease_id,
            keep_alive_handle,
            shutdown: shutdown_rx,
        };

        let join_handle = tokio::task::spawn(async move { this.run().await });

        Ok(InnerHandle {
            shutdown: shutdown_tx,
            join_handle,
        })
    }

    async fn run(mut self) -> Result<(), ExecutorError> {
        self.wait_for_running_jobs().await?;

        let mut watch_stream = self
            .client
            .watch(
                JOB_QUEUE_PREFIX.as_bytes(),
                Some(WatchOptions::new().with_prefix()),
            )
            .await
            .context(CreateWatchSnafu)?;

        // get all queued jobs on start
        let current_job_queue = self
            .client
            .get(
                JOB_QUEUE_PREFIX.as_bytes(),
                Some(GetOptions::new().with_prefix()),
            )
            .await
            .context(GetSnafu {
                key: JOB_QUEUE_PREFIX,
            })?;

        log::debug!("current_jobs: {current_job_queue:?}");

        // start watching the job queue
        watch_stream
            .request_progress()
            .await
            .context(WatchProgressSnafu)?;

        for kv in current_job_queue.kvs() {
            if let Some(job_id) = self.extract_job_id(kv).await? {
                self.run_job(job_id).await?;
            }
        }

        loop {
            tokio::select! {
                msg = watch_stream.message() => {
                    let msg = msg.context(WatchStreamSnafu)?.ok_or(ExecutorError::WatchStreamClosed)?;

                    for event in msg.events() {
                        // ignore everything but `Put` events
                        if event.event_type() != EventType::Put {
                            continue;
                        }

                        if let Some(kv) = event.kv() {

                            if kv.lease() != 0 {
                                // ignore keys that were created with a lease, these are only created when a queue
                                // entry is recreated by an executor on job execution
                                continue;
                            }

                            if let Some(job_id) = self.extract_job_id(kv).await? {
                                self.run_job(job_id).await?;
                            }

                        }
                    }

                },
                result = self.keep_alive_handle.borrow_mut() => {
                    // The etcd keep alive task exited for whatever reason, this executor will now shutdown

                    match result {
                        Ok(Err(e)) => {
                            log::error!("JobExecutors etcd lease alive task exited with error: {e}");
                            return Err(e);
                        },
                        Err(e) => {
                            log::error!("JobExecutors etcd lease alive task panicked: {e}");
                            return Err(ExecutorError::KeepAliveTaskPanicked { source: e });
                        }
                        _ => {
                            // kind of unreachable
                            return Ok(())
                        }
                    }
                },
                _ = &mut self.shutdown => {
                    self.keep_alive_handle.abort();
                    return Ok(());
                }
            }
        }
    }

    async fn run_job(&mut self, job_id: JobId) -> Result<(), ExecutorError> {
        log::debug!("running job {job_id}");
        let result = self.run_job_inner(job_id).await;

        if let Err(e) = self.clear_job_keys(job_id).await {
            // The job related keys could not be deleted. The keys are bound to this executors lease and will be removed
            // anyway when the lease expires.
            log::error!("failed to remove job with id `{job_id}` from job queue: {e:?}");

            return Err(e);
        }

        result
    }

    async fn run_job_inner(&mut self, job_id: JobId) -> Result<(), ExecutorError> {
        self.mark_job_as_running(job_id).await?;

        let mut inventory =
            self.inventory_provider
                .get_inventory()
                .await
                .context(InventorySnafu {
                    msg: "Failed to get database connection",
                })?;

        let job = inventory.get_job(job_id).await.context(InventorySnafu {
            msg: "Failed to get Job from database",
        })?;

        let job_execution = inventory
            .create_job_execution(NewJobExecution {
                job_id: job.id,
                started_at: Timestamp::now(),
                ended_at: None,
                job_status: JobStatus::Started,
            })
            .await
            .context(InventorySnafu {
                msg: "Failed to insert new JobExecution",
            })?;

        let logger =
            ExecutionLogger::create(job_execution.id, self.inventory_provider.clone()).await;

        let execution_data = JobExecutionData {
            logger: &logger,
            inventory_provider: self.inventory_provider.clone(),
            authorizer: self.authorizer.clone(),
            stop_room_backend: Arc::clone(&self.stop_room_backend),
            settings: self.settings.clone(),
            parameters: job.parameters,
            timeout: Duration::from_secs(job.timeout_secs.max(0) as u64),
            hide_duration: false,
        };

        let result = match job.kind {
            JobType::AdhocEventCleanup => execution_data.execute::<AdhocEventCleanup>().await,
            JobType::EventCleanup => execution_data.execute::<EventCleanup>().await,
            JobType::UserCleanup => execution_data.execute::<UserCleanup>().await,
            JobType::InviteCleanup => execution_data.execute::<InviteCleanup>().await,
            JobType::SelfCheck => execution_data.execute::<SelfCheck>().await,
            JobType::SyncStorageFiles => execution_data.execute::<SyncStorageFiles>().await,
            JobType::RoomCleanup => execution_data.execute::<RoomCleanup>().await,
            JobType::KeycloakAccountSync => execution_data.execute::<KeycloakAccountSync>().await,
        };

        let job_execution_update = match result {
            Ok(_) => UpdateJobExecution {
                ended_at: Some(Timestamp::now()),
                job_status: Some(JobStatus::Succeeded),
            },
            Err(_) => UpdateJobExecution {
                ended_at: Some(Timestamp::now()),
                job_status: Some(JobStatus::Failed),
            },
        };

        inventory
            .update_job_execution(job_execution.id, job_execution_update)
            .await
            .context(InventorySnafu {
                msg: "Failed to apply JobExecution update after job ended",
            })?;

        // write remaining logs and close logger
        logger.flush();

        Ok(())
    }

    /// Flags the job as running.
    ///
    /// Tags the related job queue key with this executors lease, automatically deleting it when this executor crashes
    ///
    /// Returns `false` if the job is not in the queue or already flagged as running.
    async fn mark_job_as_running(&mut self, job_id: JobId) -> Result<(), ExecutorError> {
        let queue_key = build_queue_key(job_id);
        let running_key = build_running_key(job_id);

        let txn = etcd_client::Txn::new()
            .when([
                // when the job still exists in queue
                Compare::version(queue_key.as_bytes(), CompareOp::NotEqual, 0),
                // when the job is not already flagged as running
                Compare::version(running_key.as_bytes(), CompareOp::Equal, 0),
            ])
            .and_then([
                // Add this executors lease to the queue key
                //
                // In a scenario of a successful job but a lost etcd connection, we could have the problem of a queue
                // key remaining in the list.
                // This ensures that the queue key is removed, even if the executor fails
                // to remove the key after completing the job.
                TxnOp::put(
                    queue_key.as_bytes(),
                    "",
                    Some(
                        PutOptions::new()
                            .with_ignore_value()
                            .with_lease(self.lease_id),
                    ),
                ),
                // create the running-key that flags the job as running
                TxnOp::put(
                    running_key.as_bytes(),
                    job_id.to_string(),
                    Some(PutOptions::new().with_lease(self.lease_id)),
                ),
            ]);

        self.client.txn(txn).await.context(MarkRunningSnafu)?;

        Ok(())
    }

    /// Removes the etcd keys that are related to the given job id
    async fn clear_job_keys(&mut self, job_id: JobId) -> Result<(), ExecutorError> {
        let queue_key = build_queue_key(job_id);
        let running_key = build_running_key(job_id);

        let txn = etcd_client::Txn::new().and_then([
            TxnOp::delete(queue_key.as_bytes(), None),
            TxnOp::delete(running_key.as_bytes(), None),
        ]);

        self.client.txn(txn).await.context(ClearJobSnafu)?;

        Ok(())
    }

    /// Keeps the job executers lease alive
    async fn spawn_keep_alive_task(
        client: &mut Client,
    ) -> Result<(i64, JoinHandle<Result<(), ExecutorError>>), ExecutorError> {
        let lease = client
            .lease_grant(ETCD_LEASE_TTL as i64, None)
            .await
            .context(LeaseSnafu)?;

        let response_lease_ttl = lease.ttl();

        if response_lease_ttl != ETCD_LEASE_TTL as i64 {
            log::warn!(
                "Requested lease ttl of {ETCD_LEASE_TTL} seconds, server responded with lease ttl {response_lease_ttl}"
            );
        }

        let (mut lease_keeper, _) = client
            .lease_keep_alive(lease.id())
            .await
            .context(CreateKeepAliveSnafu)?;

        let keep_alive_handle = tokio::task::spawn(async move {
            let mut keep_alive_interval =
                interval(Duration::from_secs(response_lease_ttl as u64 / 2));

            loop {
                keep_alive_interval.tick().await;

                lease_keeper.keep_alive().await.context(KeepAliveSnafu)?;
            }
        });

        Ok((lease.id(), keep_alive_handle))
    }

    /// Wait for current running keys to expire before starting a job
    async fn wait_for_running_jobs(&mut self) -> Result<(), ExecutorError> {
        let mut interval = interval(Duration::from_secs(2));

        loop {
            // check if a running key exists
            let running_keys = self
                .client
                .get(
                    JOB_RUNNING_PREFIX,
                    Some(GetOptions::new().with_prefix().with_count_only()),
                )
                .await
                .context(GetSnafu {
                    key: format!("{JOB_RUNNING_PREFIX}*"),
                })?;

            if running_keys.count() == 0 {
                return Ok(());
            }

            log::debug!(
                "Job Executor encountered a running job on startup. Waiting for the job to finish, checking again in 2 seconds"
            );

            interval.tick().await;
        }
    }

    /// Extract the jobs id from a given [`etcd_client::KeyValue`]
    ///
    /// If the job id cannot be parsed from the keys value, this function will attempt to delete the unparsable key
    /// and return [`Ok(None)`], this means that the job should be skipped.
    /// If the key could not be deleted for whatever reason, this function errors and the
    /// [`JobExecutor`] should exit.
    async fn extract_job_id(&mut self, kv: &KeyValue) -> Result<Option<JobId>, ExecutorError> {
        match parse_job_id(kv) {
            Ok(job_id) => Ok(Some(job_id)),
            Err(e) => {
                let key = kv.key_str().unwrap_or("<non_utf8>");
                log::error!("Failed to parse job id from queued job `{key}`: {e}");
                log::error!("Removing and skipping queued job `{key}`");

                self.client
                    .delete(kv.key(), None)
                    .await
                    .context(RemoveSnafu { key })?;

                Ok(None)
            }
        }
    }
}

/// Parse the job id from a key value pair
///
/// Expects the value to be a i64 job id
fn parse_job_id(kv: &KeyValue) -> Result<JobId, ExecutorError> {
    let value_str = kv.value_str().context(ParseSnafu {
        key: String::from_utf8_lossy(kv.key()),
    })?;

    value_str
        .parse::<JobId>()
        .context(ParseIntSnafu { val: value_str })
}

#[derive(Clone)]
struct JobExecutionData<'a> {
    logger: &'a ExecutionLogger,
    inventory_provider: Arc<dyn InventoryProvider>,
    authorizer: Authorizer,
    stop_room_backend: Arc<dyn StopRoomBackend>,
    settings: Arc<Settings>,
    parameters: serde_json::Value,
    timeout: Duration,
    hide_duration: bool,
}

impl JobExecutionData<'_> {
    async fn execute<J: JobImpl>(self) -> Result<(), crate::Error> {
        crate::execute::<J>(
            self.logger,
            self.inventory_provider,
            self.authorizer,
            self.stop_room_backend.as_ref(),
            &self.settings,
            self.parameters,
            self.timeout,
            self.hide_duration,
        )
        .await
    }
}

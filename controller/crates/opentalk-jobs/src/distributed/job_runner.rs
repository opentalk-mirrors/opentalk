// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{sync::Arc, time::Duration};

use opentalk_controller_api_authorization::authorization::Authorizer;
use opentalk_controller_settings::Settings;
use opentalk_controller_utils::deletion::StopRoomBackend;
use opentalk_inventory::{Inventory, InventoryProvider};
use snafu::{ResultExt, Snafu};
use tokio::{
    sync::broadcast,
    time::{Instant, interval_at},
};

use super::{
    election_task::{ElectionState, ElectionTask, ElectionTaskError, ElectionTaskHandle},
    executor_task::{ExecutorError, JobExecutorHandle},
    job_queue::{JobQueue, QueueError},
};

#[derive(Debug, Snafu)]
pub enum JobRunnerError {
    /// The election task failed with an error
    #[snafu(display("{msg}: {source}"))]
    Election {
        msg: String,
        source: ElectionTaskError,
    },

    /// The job queue task failed with an error
    #[snafu(display("Election Task exited: {source}"))]
    Queue { source: QueueError },

    /// The executor task failed with an error
    Executor { source: ExecutorError },

    /// Database error
    #[snafu(display("{msg}: {source}"))]
    Inventory {
        msg: String,
        source: opentalk_inventory::Error,
    },

    #[snafu(transparent)]
    EtcdError { source: crate::error::EtcdError },
}

/// Entry point for the controllers distributed job execution
///
/// Starts multiple related tasks:
/// - ElectionTask - Is used to elect a leader between all job executors (other controllers)
/// - JobQueue - The cron scheduler in this system, adds jobs to a distributed etcd job queue
/// - JobExecutor - A task that executes jobs that are waiting in the queue
pub struct JobRunner {
    /// Urls of the etcd cluster
    etcd_urls: Vec<String>,
    /// The inventory
    inventory_provider: Arc<dyn InventoryProvider>,
    /// Shutdown channel from the controller
    shutdown: broadcast::Receiver<()>,
    /// Handle to manage election state
    election: ElectionTaskHandle,
    /// Internal cron scheduler
    job_queue: JobQueue,
    /// Executor task handle for executing jobs
    executor: JobExecutorHandle,
}

impl JobRunner {
    /// Start the JobRunner
    pub async fn start(
        inventory_provider: Arc<dyn InventoryProvider>,
        authorizer: Authorizer,
        stop_room_backend: Arc<dyn StopRoomBackend>,
        shutdown: broadcast::Receiver<()>,
        settings: Arc<Settings>,
    ) -> Result<(), JobRunnerError> {
        log::info!("Starting JobRunner");

        let Some(etcd) = settings.etcd.as_ref() else {
            log::info!("No etcd configuration, skipping JobRunner");
            return Ok(());
        };

        let etcd_urls = Vec::from_iter(etcd.urls.iter().map(ToString::to_string));

        if etcd_urls.is_empty() {
            log::info!("Empty url list in etcd configuration, skipping JobRunner");
            return Ok(());
        }

        let election_handle =
            ElectionTask::start(etcd_urls.clone())
                .await
                .context(ElectionSnafu {
                    msg: "Failed to start job election task",
                })?;

        let job_queue = JobQueue::init(etcd_urls.clone())
            .await
            .context(QueueSnafu)?;

        let job_executor_handle = JobExecutorHandle::new(
            etcd_urls.clone(),
            inventory_provider.clone(),
            authorizer.clone(),
            stop_room_backend,
            settings.clone(),
        )
        .await;

        let mut job_runner = JobRunner {
            etcd_urls,
            inventory_provider,
            shutdown,
            election: election_handle,
            job_queue,
            executor: job_executor_handle,
        };

        tokio::spawn(async move {
            job_runner.run().await;
        });

        Ok(())
    }

    /// React to state changes from the [`ElectionTask`] and start/stop the [`JobQueue`] and `JobExecutor` accordingly.
    ///
    /// This method only returns on shutdown. In case of an error, the [`JobRunner`] enters a reconnect loop.
    async fn run(&mut self) {
        loop {
            let Err(e) = self.run_inner().await else {
                return;
            };

            log::error!("JobRunner exited with error: {e:?}");

            self.executor.stop();

            // this might error when the scheduler it is not yet initialized properly
            let _ = self.job_queue.stop().await;

            if let Err(e) = self.election.become_follower().await {
                log::error!(
                    "JobRunner failed to switch to follower state, election task died: {e}"
                );

                log::info!("Attempting to recreate election task");
                loop {
                    match ElectionTask::start(self.etcd_urls.clone()).await {
                        Ok(new_election_task_handle) => {
                            log::info!("Successfully recreated election task");
                            // This will also cause the current election handle to gracefully exit it's loop
                            // because we drop the current handle.
                            self.election = new_election_task_handle;
                            break;
                        }
                        Err(e) => {
                            let retry_in_secs = 10;

                            log::error!(
                                "Failed to recreate election task, retrying in {retry_in_secs} seconds: {e}"
                            );
                            tokio::time::sleep(Duration::from_secs(retry_in_secs)).await;
                        }
                    }
                }
            }
        }
    }

    async fn run_inner(&mut self) -> Result<(), JobRunnerError> {
        let mut inventory =
            self.inventory_provider
                .get_inventory()
                .await
                .context(InventorySnafu {
                    msg: "Failed to get database connection",
                })?;

        let mut job_sync_interval = interval_at(
            Instant::now() + Duration::from_secs(10),
            Duration::from_secs(10),
        );

        // handle the initial state
        self.handle_state_change(inventory.as_mut()).await?;

        loop {
            tokio::select! {
                _ = self.shutdown.recv() => {
                    log::info!("JobRunner got termination signal, exiting");
                    self.lost_leadership().await?;
                    return Ok(());
                }

                _ = self.election.state_changed() => {
                    self.handle_state_change(inventory.as_mut()).await?;
                }

                _ = job_sync_interval.tick()  => {
                    if self.is_leader() {
                        self.sync_job_schedules(inventory.as_mut()).await?
                    }
                }

                Err(e) = self.executor.join() => {
                    log::error!("JobExecutor exited with error {e}");

                    // The job executor task has encountered an error and exited, this runner will step down as a
                    // leader and become follower
                    self.election.become_follower().await.context(ElectionSnafu { msg: "Failed to become follower"})?;
                }
            }
        }
    }

    fn is_leader(&self) -> bool {
        *self.election.state_borrow() == ElectionState::Leader
    }

    async fn handle_state_change(
        &mut self,
        inventory: &mut dyn Inventory,
    ) -> Result<(), JobRunnerError> {
        // because the return type is !Send, this needs a new scope to make the borrow checker happy
        let election_state = { *self.election.state_borrow_and_update() };

        log::debug!("JobRunner state changed to {election_state:?}");

        match election_state {
            ElectionState::Follower | ElectionState::Hold => self.lost_leadership().await?,
            ElectionState::Leader => self.gained_leadership(inventory).await?,
        }

        Ok(())
    }

    async fn gained_leadership(
        &mut self,
        inventory: &mut dyn Inventory,
    ) -> Result<(), JobRunnerError> {
        self.executor.start().await.context(ExecutorSnafu)?;
        self.job_queue.start().await.context(QueueSnafu)?;

        self.sync_job_schedules(inventory).await?;

        Ok(())
    }

    async fn lost_leadership(&mut self) -> Result<(), JobRunnerError> {
        self.executor.stop();
        self.job_queue.stop().await.context(QueueSnafu)?;

        Ok(())
    }

    /// Updates the local jobs and their job schedule
    async fn sync_job_schedules(
        &mut self,
        inventory: &mut dyn Inventory,
    ) -> Result<(), JobRunnerError> {
        log::debug!("syncing jobs");

        let job_schedules = inventory.get_all_jobs().await.context(InventorySnafu {
            msg: "Failed to get all jobs from database",
        })?;

        let mut job_list = self.job_queue.current_jobs();

        for job in job_schedules {
            let _ = job_list.remove(&job.id);

            self.job_queue
                .add_or_update(job)
                .await
                .context(QueueSnafu)?;
        }

        // All remaining jobs are considered stale
        for job_id in job_list {
            if let Some(job_info) = self.job_queue.remove(job_id).await.context(QueueSnafu)? {
                log::debug!(
                    "Removing stale job (id: {}, name: {}, kind: {}, cron schedule: {}",
                    job_info.job.id,
                    job_info.job.name,
                    job_info.job.kind,
                    job_info.job.recurrence
                );
            }
        }

        Ok(())
    }
}

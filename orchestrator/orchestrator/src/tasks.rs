// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use anyhow::{Context, Result};
use tokio::{
    sync::broadcast,
    task::JoinSet,
    time::{Instant, timeout_at},
};

const APPLICATION_SHUTDOWN_GRACE_PERIOD: Duration = Duration::from_secs(3);

pub struct ShutdownReceiver(pub broadcast::Receiver<()>);

impl ShutdownReceiver {
    pub async fn wait_for_shutdown(&mut self) {
        let res = self.0.recv().await;
        if let Err(e) = res {
            tracing::error!("Shutdown sender was dropped: {e}");
        }
    }

    pub fn resubscribe(&self) -> ShutdownReceiver {
        ShutdownReceiver(self.0.resubscribe())
    }
}

/// Returned by managed tasks
struct TaskFinished {
    /// The task name, used for logging purposes
    name: &'static str,
    /// The result of the task
    result: Result<()>,
}

/// Manages the lifecycle of all tasks in the orchestrator.
///
/// When one of the task finishes for whatever reason, all other tasks are signaled to shut down.
pub struct Tasks {
    /// The shutdown sender to signal all tasks to shut down
    shutdown_signal: broadcast::Sender<()>,
    /// The join set for all tasks managed tasks
    tasks: JoinSet<TaskFinished>,
}

impl Tasks {
    /// Creates a new [Tasks] instance
    pub fn new() -> Self {
        let (shutdown_signal, _) = broadcast::channel(1);

        Self {
            shutdown_signal,
            tasks: JoinSet::new(),
        }
    }

    /// Spawns a new task with the given name
    ///
    /// The task is given a receiver that will signal when the application is shutting down. The
    /// task must listen to this signal and shutdown gracefully when it is received.
    pub fn spawn<F, Fut>(&mut self, name: &'static str, task: F)
    where
        F: FnOnce(ShutdownReceiver) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        tracing::debug!("Spawning {name} task");
        let shutdown_signal = self.shutdown_signal.subscribe();

        self.tasks.spawn(async move {
            let result = task(ShutdownReceiver(shutdown_signal)).await;
            TaskFinished { name, result }
        });
    }

    /// Waits for the first task to finish and then signals all other tasks to shutdown.
    ///
    /// If the shutdown signal cannot be sent, all tasks are aborted.
    pub async fn wait_for_shutdown(&mut self) -> Result<()> {
        match self.tasks.join_next().await {
            Some(Ok(TaskFinished { name, result })) => match result {
                Ok(()) => tracing::debug!("{name} task finished"),
                Err(e) => tracing::error!("{name} exited with error: {e:?}"),
            },
            Some(Err(e)) => {
                tracing::error!("A task panicked: {e}");
            }
            None => {
                anyhow::bail!("Failed to spawn any task");
            }
        }

        if let Err(e) = self.shutdown_gracefully().await {
            tracing::error!("Graceful shutdown failed, aborting all tasks");
            self.tasks.abort_all();
            return Err(e.context("Forced shutdown"));
        }

        Ok(())
    }

    async fn shutdown_gracefully(&mut self) -> anyhow::Result<()> {
        tracing::info!("Shutting down remaining tasks...");
        self.shutdown_signal
            .send(())
            .context("Failed to send shutdown signal")?;

        let deadline = Instant::now() + APPLICATION_SHUTDOWN_GRACE_PERIOD;

        loop {
            let result = timeout_at(deadline, self.tasks.join_next()).await;
            match result {
                Ok(None) => {
                    tracing::info!("All tasks exited gracefully");
                    return Ok(());
                }
                Ok(Some(Ok(TaskFinished { name, result }))) => match result {
                    Ok(()) => tracing::info!("{name} task exited"),
                    Err(e) => tracing::error!("{name} task exited with error: {e:?}"),
                },
                Ok(Some(Err(e))) => {
                    tracing::error!("A task panicked: {e}");
                }
                Err(_) => {
                    anyhow::bail!("Not all tasks finished within the grace period");
                }
            }
        }
    }
}

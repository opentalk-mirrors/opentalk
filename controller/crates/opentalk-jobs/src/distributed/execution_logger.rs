// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{sync::Arc, time::Duration};

use log::{Log, Metadata, Record};
use opentalk_inventory::{
    InventoryProvider, JobExecutionId, JobExecutionLogLevel as LogLevel, NewJobExecutionLog,
};
use opentalk_types_common::time::Timestamp;
use snafu::{ResultExt, Snafu};
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    time::interval,
};

#[derive(Debug, Snafu)]
pub enum ExecutionLoggerError {
    #[snafu(display("Failed to write log buffer to database: {source}"))]
    BatchInsertFailed { source: opentalk_inventory::Error },

    #[snafu(display("Failed to get inventory: {source}"))]
    FailedToGetInventory { source: opentalk_inventory::Error },
}

/// A logger that writes log messages to the database
///
/// Each job needs to create their own logger instance.
#[derive(Clone)]
pub struct ExecutionLogger {
    /// The execution id of the related job
    execution_id: JobExecutionId,
    /// A sender to signal the that no further messages will be logged and the pending messages can be written to the
    /// database.
    flush_sender: UnboundedSender<()>,
    /// Used to send logs to the database write task. Behaving like a buffer for pending log messages
    ///
    /// This channel is closed when a database error occurs.
    log_sender: UnboundedSender<NewJobExecutionLog>,
}

impl ExecutionLogger {
    pub async fn create(
        execution_id: JobExecutionId,
        inventory_provider: Arc<dyn InventoryProvider>,
    ) -> Self {
        let (log_sender, log_receiver) = mpsc::unbounded_channel();
        let (flush_sender, flush_receiver) = mpsc::unbounded_channel();

        // start the task that does the actual database writing
        LoggerTask::start(
            execution_id,
            log_receiver,
            flush_receiver,
            inventory_provider,
        )
        .await;

        Self {
            execution_id,
            flush_sender,
            log_sender,
        }
    }
}

impl Log for ExecutionLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // The ExecutionLogger has the same log level as our global controller logger
        log::logger().enabled(metadata)
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let log_level = match record.level() {
            log::Level::Error => LogLevel::Error,
            log::Level::Warn => LogLevel::Warn,
            log::Level::Info => LogLevel::Info,
            log::Level::Debug => LogLevel::Debug,
            log::Level::Trace => LogLevel::Trace,
        };

        let log = NewJobExecutionLog {
            execution_id: self.execution_id,
            logged_at: Timestamp::now(),
            log_level,
            log_message: record.args().to_string(),
        };

        // ignore the result, there is nothing that can be done if the write task exited
        let _ = self.log_sender.send(log);
    }

    fn flush(&self) {
        // ignore the result, there is nothing that can be done if the write task exited
        let _ = self.flush_sender.send(());
    }
}

/// The LoggerTask is responsible for writing the logs to the database.
///
/// When the database operation fails, the task will fall back to stdout logging.
struct LoggerTask {
    /// The id of the job execution
    execution_id: JobExecutionId,
    /// Receiver for log messages
    log_receiver: UnboundedReceiver<NewJobExecutionLog>,
    /// Receiver for the flush command
    flush_receiver: UnboundedReceiver<()>,

    inventory_provider: Arc<dyn InventoryProvider>,
}

impl LoggerTask {
    /// Starts the logger task
    async fn start(
        execution_id: JobExecutionId,
        log_receiver: UnboundedReceiver<NewJobExecutionLog>,
        flush_receiver: UnboundedReceiver<()>,
        inventory_provider: Arc<dyn InventoryProvider>,
    ) {
        let this = Self {
            execution_id,
            log_receiver,
            flush_receiver,
            inventory_provider,
        };

        this.spawn_write_task();
    }

    /// Spawns the write task that receives log messages through an unbounded channel
    ///
    /// The channel acts as a buffer for log messages. The messages are then bulk inserted in a fixed interval.
    ///
    /// When writing to the database fails, the log messages will be printed to the stdout.
    fn spawn_write_task(mut self) {
        tokio::spawn(async move {
            let mut write_interval = interval(Duration::from_secs(3));

            let mut buffer: Vec<NewJobExecutionLog> = vec![];

            loop {
                tokio::select! {
                    _ = write_interval.tick() => {
                        if self.log_receiver.recv_many(&mut buffer, 50).await == 0 {
                            // returned zero -> sender closed, the write task can exit
                            break;
                        }

                        if let Err(e) = self.write_to_db(&mut buffer).await  {
                            log::error!("Unrecoverable error while writing execution logs for job {}, logs will be lost: {:?}", self.execution_id, e);

                            self.stdout_fallback(buffer).await;
                            return
                        }
                    }
                    Some(_) = self.flush_receiver.recv() => {
                        if let Err(e) = self.flush_log_receiver(&mut buffer).await {
                            log::error!("Unrecoverable error while flushing execution logs for job {}, logs will be lost: {:?}", self.execution_id, e);
                        }
                        break;
                    }
                }
            }
        });
    }

    /// Prints log output to stdout until this logger is destroyed
    ///
    /// This also prints all buffered logs to stdout.
    async fn stdout_fallback(mut self, buffer: Vec<NewJobExecutionLog>) {
        if !buffer.is_empty() {
            log::info!(
                "Flushing log buffer for execution '{}' to stdout, log message timestamps will be inaccurate",
                self.execution_id
            );

            for msg in buffer {
                self.write_to_stdout(msg)
            }

            log::info!("Flush for execution `{}` complete", self.execution_id);
        }

        while let Some(msg) = self.log_receiver.recv().await {
            self.write_to_stdout(msg)
        }
    }

    fn write_to_stdout(&self, msg: NewJobExecutionLog) {
        let level = match msg.log_level {
            LogLevel::Trace => log::Level::Trace,
            LogLevel::Debug => log::Level::Debug,
            LogLevel::Info => log::Level::Info,
            LogLevel::Warn => log::Level::Warn,
            LogLevel::Error => log::Level::Error,
        };

        let target = &format!("execution_id:{}", self.execution_id);

        let metadata = Metadata::builder().level(level).target(target).build();

        // Dirty little hack to work around https://github.com/rust-lang/rust/issues/92698
        #[expect(irrefutable_let_patterns)]
        if let args = format_args!("{}", msg.log_message) {
            let record = Record::builder().metadata(metadata).args(args).build();

            log::logger().log(&record);
        }
    }

    async fn write_to_db(
        &self,
        buffer: &mut Vec<NewJobExecutionLog>,
    ) -> Result<(), ExecutionLoggerError> {
        let mut inventory = self
            .inventory_provider
            .get_inventory()
            .await
            .context(FailedToGetInventorySnafu)?;

        inventory
            .create_job_execution_logs(buffer)
            .await
            .context(BatchInsertFailedSnafu)?;

        // clear the buffer if logs were successfully written to db
        buffer.clear();

        Ok(())
    }

    async fn flush_log_receiver(
        &mut self,
        buffer: &mut Vec<NewJobExecutionLog>,
    ) -> Result<(), ExecutionLoggerError> {
        // closing the receiver to prevent race conditions while draining remaining messages
        self.log_receiver.close();

        while self.log_receiver.recv_many(buffer, 50).await != 0 {
            self.write_to_db(buffer).await?;
        }

        Ok(())
    }
}

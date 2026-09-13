// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use anyhow::{Context as _, Result};
use futures::stream::StreamExt;
use lapin::{message::Delivery, options::BasicRejectOptions};
use opentalk_mail_worker_protocol as proto;
use service_probe::{ServiceState, set_service_state};

use crate::{
    mail::{MailBuilder, MailTemplate},
    rabbitmq::RabbitMqService,
};

/// A Mail Worker
pub struct Worker<'a, T> {
    mail_backend: &'a T,
    mail_builder: &'a MailBuilder,
    task_processing_timeout: Option<Duration>,
}

impl<'a, T> Worker<'a, T>
where
    T: lettre::AsyncTransport<Error = lettre::transport::smtp::Error> + Sync,
{
    /// Creates a new Worker instance
    pub fn new(
        mail_builder: &'a MailBuilder,
        mail_backend: &'a T,
        task_processing_timeout: Option<Duration>,
    ) -> Self {
        Self {
            mail_backend,
            mail_builder,
            task_processing_timeout,
        }
    }

    /// Run the worker loop.
    ///
    /// Doesn't return a result, instead it will just process all pending
    /// messages, and log any errors that it encounters. If processing the tasks
    /// fails at any point, the error will be logged and the function returns.
    pub async fn run(&self, mut rabbitmq_service: RabbitMqService) {
        set_service_state(ServiceState::Ready);

        log::info!("Worker started");

        log::debug!("Waiting for next task from RabbitMQ");
        while let Some(delivery) = rabbitmq_service.consumer.next().await {
            log::trace!("Received new RabbitMQ message delivery");

            let Ok(delivery) = delivery.inspect_err(|e| {
                log::warn!("Error reading RabbitMQ message delivery: {e}");
            }) else {
                return;
            };

            let Ok(()) = self
                .handle_rabbitmq_delivery(delivery)
                .await
                .inspect_err(|_e| {
                    log::error!("Shutting down worker because of error");
                })
            else {
                return;
            };
            log::debug!("Waiting for next task from RabbitMQ");
        }

        log::error!("Worker was disconnected from the RabbitMQ channel.");
    }

    /// Process a received delivery.
    async fn handle_rabbitmq_delivery(&self, delivery: Delivery) -> Result<()> {
        log::debug!("Received new mail task");

        let data = &delivery.data;

        let result = if let Some(timeout) = self.task_processing_timeout {
            let Ok(mail_task_result) =
                tokio::time::timeout(timeout, self.handle_mail_task(data)).await
            else {
                log::warn!(
                    "Attempt to send mail notification timed out after {timeout:?}, rejecting the task to RabbitMQ for requeue and continuing"
                );

                delivery
                    .reject(BasicRejectOptions { requeue: true })
                    .await
                    .context("Error acknowledging delivery to RabbitMQ")?;
                return Ok(());
            };
            mail_task_result
        } else {
            self.handle_mail_task(data).await
        };

        if let Err(e) = result {
            log::error!("Error sending E-Mail: {e}");

            let requeue = if let Some(smtp_error) =
                e.downcast_ref::<lettre::transport::smtp::Error>()
            {
                smtp_error.is_timeout() || (!smtp_error.is_permanent() && !smtp_error.is_client())
            } else {
                false
            };

            if let Err(e) = delivery.reject(BasicRejectOptions { requeue }).await {
                log::error!("Error rejecting the delivery to RabbitMQ: {e}");
            }

            return Ok(());
        }

        if let Err(e) = delivery.ack(Default::default()).await {
            log::error!("Error acknowledging processed message to RabbitMQ: {e}");
        }

        Ok(())
    }

    async fn handle_mail_task(&self, data: &[u8]) -> Result<()> {
        let de = &mut serde_json::Deserializer::from_slice(data);
        let versioned_message: proto::MailTask = serde_path_to_error::deserialize(de)?;

        match versioned_message {
            proto::MailTask::V1(message) => {
                send_mail_v1(self.mail_backend, self.mail_builder, &message).await?
            }
        }
        Ok(())
    }
}

pub async fn send_mail_v1<T>(
    mail_backend: &T,
    mail_builder: &MailBuilder,
    message: &proto::v1::Message,
) -> Result<()>
where
    T: lettre::AsyncTransport<Error = lettre::transport::smtp::Error> + Sync,
{
    let email = mail_builder.generate_email(message)?;
    // Hack until lettre supports printing the To header
    let to = message.generate_to_mbox(mail_builder);
    mail_backend.send(email).await?;

    log::info!(
        "Mail sent to {}",
        to.map(|mailbox| mailbox.to_string())
            .unwrap_or_else(|_| "N/A".to_string())
    );

    Ok(())
}

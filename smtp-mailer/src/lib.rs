// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use anyhow::{Context, Result};
use lettre::{AsyncSmtpTransport, Tokio1Executor, transport::smtp::authentication::Credentials};
use log::debug;
use service_probe::{ServiceState, set_service_state, start_probe};
use settings::MonitoringSettings;

use crate::{settings::RabbitMqConfig, worker::Worker};

const TIME_BETWEEN_RABBITMQ_CONNECTION_ATTEMPTS: Duration = Duration::from_secs(1);

pub mod i18n;
pub(crate) mod mail;

mod ics;
pub mod preview;
mod rabbitmq;
pub mod settings;
mod worker;

pub use mail::{MailBuilder, MailTemplate};
pub use worker::send_mail_v1;

/// Entry point of the library part of smtp-mailer
pub async fn run(settings: settings::Settings) -> Result<()> {
    if let Some(MonitoringSettings { port, addr }) = settings.monitoring {
        start_probe(addr, port, ServiceState::Up).await?;
    }

    let mail_builder = MailBuilder::new(&settings)?;
    let smtp_client: AsyncSmtpTransport<Tokio1Executor> =
        settings.smtp.smtp_server.clone().try_into()?;
    let task_processing_timeout = match settings.rabbit_mq.task_processing_timeout_seconds {
        0 => None,
        seconds => Some(Duration::from_secs(seconds)),
    };

    loop {
        let next_connection_attempt =
            std::time::Instant::now() + TIME_BETWEEN_RABBITMQ_CONNECTION_ATTEMPTS;
        run_worker(
            &settings.rabbit_mq,
            &mail_builder,
            &smtp_client,
            task_processing_timeout,
        )
        .await;
        tokio::time::sleep_until(next_connection_attempt.into()).await;
    }
}

async fn run_worker(
    rabbitmq_settings: &RabbitMqConfig,
    mail_builder: &MailBuilder,
    smtp_client: &AsyncSmtpTransport<Tokio1Executor>,
    task_processing_timeout: Option<Duration>,
) {
    debug!("Connecting to RabbitMQ channel…");
    let Ok(rabbitmq_service) = crate::rabbitmq::RabbitMqService::new(rabbitmq_settings)
        .await
        .inspect_err(|e| log::warn!("Couldn't connect to RabbitMQ channel: {e}"))
    else {
        return;
    };
    debug!(
        "Connection to RabbitMQ channel established, starting processing of mail notification tasks"
    );

    let worker = Worker::new(mail_builder, smtp_client, task_processing_timeout);

    worker.run(rabbitmq_service).await;
    set_service_state(ServiceState::Up);
}

impl TryFrom<settings::SmtpUri> for AsyncSmtpTransport<Tokio1Executor> {
    type Error = anyhow::Error;
    fn try_from(val: settings::SmtpUri) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
        let host = val.host().context("SMTP Host")?;

        let mut builder = match val.scheme() {
            "smtp" => {
                if val.disable_starttls() {
                    AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host)
                } else {
                    AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)?
                }
            }
            "smtps" => AsyncSmtpTransport::<Tokio1Executor>::relay(host)?,
            _ => unreachable!(),
        };

        match val.credentials()? {
            (Some(u), None) => {
                let creds = Credentials::new(u, "".to_owned());
                builder = builder.credentials(creds);
            }
            (Some(u), Some(p)) => {
                let creds = Credentials::new(u, p);
                builder = builder.credentials(creds);
            }
            _ => {}
        }

        if let Some(port) = val.port() {
            builder = builder.port(port);
        }
        Ok(builder.build())
    }
}

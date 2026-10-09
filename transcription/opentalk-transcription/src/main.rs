// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! OpenTalk Transcription service.
//!
//! This binary joins OpenTalk conference rooms, receives the participants' audio
//! tracks via LiveKit and transcribes their speech with a local whisper.cpp
//! model. The resulting transcript segments are sent back to the OpenTalk
//! controller.
//!
//! [`main`] wires together the configuration, logging, the HTTP API, the
//! optional orchestrator client and the per-room transcription sessions, and
//! drives their shared lifecycle through [`tasks::Tasks`].

use std::{collections::HashMap, process::ExitCode, sync::Arc};

use anyhow::{Context, Result};
use opentalk_orchestrator_client::ServiceAddress;
use service_probe::{ServiceState, start_probe, stop_probe};
use settings::Settings;
use tokio::sync::Mutex;
use whisper_rs::install_logging_hooks;

use crate::{cli::Commands, orchestrator::OrchestratorStateProvider, tasks::Tasks};

mod app;
mod audio;
mod audio_processing;
mod cli;
mod commands;
mod metrics;
mod orchestrator;
mod server;
mod settings;
mod signal;
mod tasks;
mod trace;
mod transcription;

pub(crate) use app::{AppState, TranscriptionTasks};

#[tokio::main]
async fn main() -> Result<ExitCode> {
    // Capture logs emitted before initializing the logger from the settings.
    let interim_logger = trace::bootstrap();

    let args = cli::parse_args();
    if args.should_print() {
        return Ok(ExitCode::SUCCESS);
    }

    let settings =
        Arc::new(Settings::load(args.config.as_deref()).context("Failed to read config")?);

    let tracer_provider = trace::init(interim_logger, settings.tracing.as_ref())?;

    install_logging_hooks();

    if let Some(Commands::Health { endpoint }) = args.command {
        return commands::run_health_command(endpoint, &settings).await;
    }

    if let Some(monitoring) = &settings.monitoring {
        start_probe(monitoring.addr, monitoring.port, ServiceState::Up)
            .await
            .context("Failed to start monitoring endpoint")?;
    }

    let mut tasks = Tasks::new();

    tasks.spawn("shutdown_signal_handler", |shutdown| {
        signal::run_shutdown_signal_handler(shutdown)
    });

    let transcription_tasks: TranscriptionTasks = Arc::new(Mutex::new(HashMap::new()));

    let mut orchestrator_handle = None;

    if let Some(orchestrator_config) = &settings.orchestrator {
        let transcription_key_ids = settings
            .http
            .api_keys
            .inner()
            .iter()
            .map(|api_key| api_key.id.clone());

        let state_provider = OrchestratorStateProvider::new(Arc::clone(&transcription_tasks));

        let (client, handle) = opentalk_orchestrator_client::OrchestratorClient::create(
            orchestrator_config.clone(),
            transcription_key_ids,
        )
        .await;
        orchestrator_handle = Some(handle);

        let client_address = ServiceAddress::Port(settings.http.port);

        tasks.spawn("orchestrator_client", move |mut shutdown| async move {
            client
                .run(client_address, state_provider, async move {
                    let _ = shutdown.wait_for_shutdown().await;
                })
                .await
        });
    }

    tasks.spawn("webserver", move |shutdown| {
        server::run_web_server(
            Arc::clone(&settings),
            Arc::clone(&transcription_tasks),
            orchestrator_handle,
            shutdown,
        )
    });

    tasks.wait_for_shutdown().await?;
    stop_probe().await;

    opentalk_tracing::shutdown(tracer_provider);

    Ok(ExitCode::SUCCESS)
}

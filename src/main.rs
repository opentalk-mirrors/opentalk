// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};
use axum::{
    Router,
    extract::{Json, State},
    routing::{get, post},
};
use futures::future::join_all;
use opentalk_client_signaling::{
    AuthConfig, ControllerConfig, OpenTalkClient,
    config::{ClientConfig, ClientCredentialsAuthConfig},
    types::signaling::transcription::http::StartTranscriptionRequestBody,
};
use service_probe::{ServiceState, set_service_state, start_probe, stop_probe};
use settings::Settings;
use tokio::{
    net::TcpListener,
    select,
    signal::{
        self,
        unix::{SignalKind, signal},
    },
    sync::Mutex,
    task::JoinHandle,
    time::timeout,
};
use transcription::TranscriptionSession;
use whisper_rs::{WhisperContext, WhisperContextParameters, install_logging_hooks};

use crate::tasks::{ShutdownReceiver, Tasks};

mod audio;
mod cli;
mod metrics;
mod settings;
mod tasks;
mod transcription;
mod voice_activity_detection;

pub(crate) struct AppState {
    pub(crate) tasks: Arc<Mutex<Vec<JoinHandle<Result<()>>>>>,
    pub(crate) settings: Arc<Settings>,
    pub(crate) shutdown: ShutdownReceiver,
    pub(crate) whisper_context: Arc<WhisperContext>,
    pub(crate) prometheus_state: Arc<metrics::PrometheusState>,
    pub(crate) opentalk_client: Arc<OpenTalkClient>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            tasks: self.tasks.clone(),
            settings: self.settings.clone(),
            shutdown: self.shutdown.resubscribe(),
            whisper_context: self.whisper_context.clone(),
            prometheus_state: self.prometheus_state.clone(),
            opentalk_client: self.opentalk_client.clone(),
        }
    }
}

impl AppState {
    pub(crate) async fn new(
        settings: Arc<Settings>,
        shutdown_signal: ShutdownReceiver,
    ) -> Result<Self> {
        let opentalk_client = OpenTalkClient::create(ClientConfig {
            auth: AuthConfig::ClientCredentials(ClientCredentialsAuthConfig {
                issuer: settings.auth.issuer.clone(),
                client_id: settings.auth.client_id.clone(),
                client_secret: settings.auth.client_secret.clone(),
            }),
            controller: ControllerConfig {
                domain: settings.controller.domain.clone(),
                insecure: settings.controller.insecure,
            },
        })
        .await?;

        let whisper_context = WhisperContext::new_with_params(
            &settings.transcription.model,
            WhisperContextParameters::default(),
        )
        .context("Failed to create whisper context")?;

        Ok(Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
            settings: settings.clone(),
            shutdown: shutdown_signal,
            whisper_context: Arc::new(whisper_context),
            prometheus_state: Arc::new(metrics::PrometheusState::new()),
            opentalk_client: Arc::new(opentalk_client),
        })
    }

    pub(crate) async fn spawn_session(
        &self,
        room_info: StartTranscriptionRequestBody,
    ) -> Result<()> {
        let context = Arc::new(self.clone());
        log::debug!("Start Transcription session {:?}", room_info.room_id);
        let mut session = TranscriptionSession::create(context, room_info)
            .await
            .context("transcription session failed to start")?;

        let task = tokio::spawn(async move {
            if let Err(ref error) = session.run().await {
                log::error!("transcription session failed:\n{error:?}",);
            }

            Ok(())
        });

        self.tasks.lock().await.push(task);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    install_logging_hooks();

    let args = cli::parse_args();
    if !args.should_start() {
        return Ok(());
    }

    let settings = Arc::new(Settings::load(args.config.as_ref()).context("Failed to read config")?);

    start_probe(
        settings.monitoring.addr,
        settings.monitoring.port,
        ServiceState::Up,
    )
    .await
    .context("Failed to start monitoring endpoint")?;

    let mut tasks = Tasks::new();

    tasks.spawn("shutdown_signal_handler", |shutdown| async move {
        run_shutdown_signal_handler(shutdown).await;
        Ok(())
    });

    tasks.spawn("webserver", move |shutdown| {
        run_web_server(settings.clone(), shutdown)
    });

    tasks.wait_for_shutdown().await?;
    stop_probe().await;

    Ok(())
}

async fn run_web_server(settings: Arc<Settings>, mut shutdown: ShutdownReceiver) -> Result<()> {
    let addr = settings.http.addr;
    let port = settings.http.port;

    let transcription_model_name = settings.transcription.model.clone();

    let app_state = AppState::new(settings.clone(), shutdown.resubscribe()).await?;
    let prometheus_state = app_state.prometheus_state.clone();

    // TODO: Add Metrics endpoint
    // TODO: Add bearer token verification
    let app = Router::new()
        .route("/start", post(start_transcription))
        .route("/metrics", get(async move || prometheus_state.metrics()))
        .route(
            "/model",
            get(async move || transcription_model_name.clone()),
        )
        .with_state(app_state.clone());

    let listener = TcpListener::bind((addr, port)).await?;

    set_service_state(ServiceState::Ready);

    let result = axum::serve(listener, app)
        .with_graceful_shutdown(async move { shutdown.wait_for_shutdown().await })
        .await;

    // Settings the service state from `Ready` to `Up` after the web server has stopped
    set_service_state(ServiceState::Up);

    if let Err(e) = result {
        log::error!("Web server exited with error: {e}");
    }

    // Wait for all remaining transcription tasks to finish
    let mut transcription_tasks = app_state.tasks.lock().await;
    transcription_tasks.retain(|task| !task.is_finished());

    if !transcription_tasks.is_empty() {
        log::info!(
            "waiting for {} remaining transcription tasks to finish",
            transcription_tasks.len()
        );

        match timeout(
            Duration::from_secs(10),
            join_all(transcription_tasks.iter_mut()),
        )
        .await
        {
            Ok(_) => log::info!("All transcription tasks are finished"),
            Err(_) => log::error!("Timeout while waiting for transcription tasks to finish"),
        }
    } else {
        log::info!("All transcription tasks are finished");
    }

    Ok(())
}

async fn start_transcription(
    State(transcription): State<AppState>,
    Json(room_info): Json<StartTranscriptionRequestBody>,
) {
    transcription.spawn_session(room_info).await.unwrap();
}

async fn run_shutdown_signal_handler(mut shutdown_signal: ShutdownReceiver) {
    let mut sig_term = signal(SignalKind::terminate()).expect("cannot setup SIGTERM handler");
    select! {
        _ = signal::ctrl_c() => { log::debug!("received Ctrl-C"); }
        _ = sig_term.recv() => { log::debug!("received SIGTERM"); }
        _ = shutdown_signal.wait_for_shutdown() => {
            log::trace!("Shutdown handler received shutdown signal from application state");
            return;
        }
    }

    log::info!("Received shutdown signal...");
}

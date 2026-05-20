// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{HashMap, hash_map::Entry},
    fs,
    sync::Arc,
    time::Duration,
};

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use axum::{Router, routing::get};
use futures::future::join_all;
use opentalk_client_signaling::{AuthConfig, OpenTalkClient, config::ClientConfig};
use opentalk_orchestrator_client::{
    ServiceAddress, TranscriptionEvent, TranscriptionResource, client::OrchestratorHandle,
};
use opentalk_transcription_web_api::v1::{self, TranscriptionBackend, TranscriptionTarget};
use opentalk_types_api_common::error::ApiError;
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

use crate::{
    orchestrator::OrchestratorStateProvider,
    tasks::{ShutdownReceiver, Tasks},
};

mod audio;
mod cli;
mod metrics;
mod orchestrator;
mod settings;
mod tasks;
mod transcription;
mod voice_activity_detection;

pub(crate) type TranscriptionTasks =
    Arc<Mutex<HashMap<TranscriptionTarget, JoinHandle<Result<()>>>>>;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) tasks: TranscriptionTasks,
    pub(crate) whisper_context: Arc<WhisperContext>,
    pub(crate) prometheus_state: Arc<metrics::PrometheusState>,
    pub(crate) opentalk_client: Arc<OpenTalkClient>,
    pub(crate) orchestrator_handle: Option<OrchestratorHandle>,
    pub(crate) shutdown: ShutdownReceiver,
}

impl AppState {
    pub(crate) async fn new(
        settings: Arc<Settings>,
        transcription_tasks: TranscriptionTasks,
        orchestrator_handle: Option<OrchestratorHandle>,
        shutdown_signal: ShutdownReceiver,
    ) -> Result<Self> {
        let opentalk_client = OpenTalkClient::create(ClientConfig {
            auth: AuthConfig::ApiKey(settings.controller.api_key.clone()),
            controller: settings.controller.url.clone(),
        })
        .await?;

        if !fs::exists(&settings.transcription.model)
            .context("Failed to check if model path is valid")?
        {
            bail!(
                "Failed to load model '{}', file does not exist",
                settings.transcription.model
            )
        }

        let whisper_context = WhisperContext::new_with_params(
            &settings.transcription.model,
            WhisperContextParameters::default(),
        )
        .context("Failed to create whisper context")?;

        Ok(Self {
            tasks: transcription_tasks,
            whisper_context: Arc::new(whisper_context),
            prometheus_state: Arc::new(metrics::PrometheusState::new()),
            opentalk_client: Arc::new(opentalk_client),
            orchestrator_handle,
            shutdown: shutdown_signal,
        })
    }
}

#[async_trait]
impl TranscriptionBackend for AppState {
    async fn init(&self, target: TranscriptionTarget) -> Result<(), ApiError> {
        let context = Arc::new(self.clone());
        log::debug!("Start Transcription session {:?}", target.room_id);
        let session = TranscriptionSession::create(context, target.clone())
            .await
            .map_err(|err| {
                log::error!("Failed to create transcription session: {err:?}");

                ApiError::internal()
            })?;

        let tasks = Arc::clone(&self.tasks);
        let orchestrator_handle = self.orchestrator_handle.clone();
        let transcription_target = target.clone();

        let task = tokio::spawn(async move {
            match session.run().await {
                Ok(()) => {
                    log::debug!(
                        "Transcription session for room {} finished",
                        transcription_target.room_id
                    );
                }
                Err(e) => {
                    log::error!(
                        "Transcription session for room {} exited with error: {e:?}",
                        transcription_target.room_id
                    );
                }
            }

            if tasks.lock().await.remove(&transcription_target).is_none() {
                log::error!(
                    "Transcription session for room {:?} was not found in the task list",
                    transcription_target.room_id
                );
            }

            if let Some(orchestrator_handle) = orchestrator_handle {
                if let Err(e) = orchestrator_handle
                    .send_event(TranscriptionEvent::RemoveTranscription(
                        TranscriptionResource {
                            room_id: transcription_target.room_id,
                            breakout_id: transcription_target.breakout_room,
                        },
                    ))
                    .await
                {
                    log::error!(
                        "Failed to notify orchestrator about removed transcription session: {e:?}"
                    );
                }
            }

            Ok(())
        });

        let mut tasks = self.tasks.lock().await;

        match tasks.entry(target) {
            Entry::Occupied(occupied_entry) => {
                log::error!(
                    "Received request to start transcription for room {:?} but a session is already running",
                    occupied_entry.key().room_id
                );
                task.abort();

                return Err(ApiError::conflict()
                    .with_message("A transcription session is already running for this room"));
            }
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(task);
            }
        }

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

    if let Some(monitoring) = &settings.monitoring {
        start_probe(monitoring.addr, monitoring.port, ServiceState::Up)
            .await
            .context("Failed to start monitoring endpoint")?;
    }

    let mut tasks = Tasks::new();

    tasks.spawn("shutdown_signal_handler", |shutdown| async move {
        run_shutdown_signal_handler(shutdown).await;
        Ok(())
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
        run_web_server(
            Arc::clone(&settings),
            Arc::clone(&transcription_tasks),
            orchestrator_handle,
            shutdown,
        )
    });

    tasks.wait_for_shutdown().await?;
    stop_probe().await;

    Ok(())
}

async fn run_web_server(
    settings: Arc<Settings>,
    transcription_tasks: TranscriptionTasks,
    orchestrator_handle: Option<OrchestratorHandle>,
    mut shutdown: ShutdownReceiver,
) -> Result<()> {
    let addr = settings.http.addr;
    let port = settings.http.port;

    let transcription_model_name = settings.transcription.model.clone();

    let app_state = AppState::new(
        settings.clone(),
        transcription_tasks,
        orchestrator_handle,
        shutdown.resubscribe(),
    )
    .await?;
    let prometheus_state = app_state.prometheus_state.clone();

    let auth_middleware = settings
        .http
        .api_keys
        .auth_middleware()
        .context("Invalid API key configuration")?;

    let app = Router::new()
        .merge(v1::routes().layer(auth_middleware))
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
    transcription_tasks.retain(|_, task| !task.is_finished());

    if !transcription_tasks.is_empty() {
        log::info!(
            "waiting for {} remaining transcription tasks to finish",
            transcription_tasks.len()
        );

        match timeout(
            Duration::from_secs(10),
            join_all(transcription_tasks.iter_mut().map(|(_, task)| task)),
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

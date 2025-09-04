// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{net::IpAddr, sync::Arc};

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
use service_probe::{ServiceState, set_service_state, start_probe};
use settings::Settings;
use tokio::{
    net::TcpListener,
    select,
    signal::{
        ctrl_c,
        unix::{SignalKind, signal},
    },
    sync::{Mutex, broadcast},
    task::JoinHandle,
};
use transcription::TranscriptionSession;
use whisper_rs::{WhisperContext, WhisperContextParameters, install_logging_hooks};

mod audio;
mod cli;
mod metrics;
mod settings;
mod transcription;

pub(crate) struct AppState {
    pub(crate) tasks: Arc<Mutex<Vec<JoinHandle<Result<()>>>>>,
    pub(crate) settings: Arc<Settings>,
    pub(crate) shutdown: broadcast::Receiver<()>,
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
    )?;

    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

    let transcription = AppState {
        tasks: Arc::new(Mutex::new(Vec::new())),
        settings: settings.clone(),
        shutdown: shutdown_rx,
        whisper_context: Arc::new(whisper_context),
        prometheus_state: Arc::new(metrics::PrometheusState::new()),
        opentalk_client: Arc::new(opentalk_client),
    };

    start_probe(
        settings.monitoring.addr,
        settings.monitoring.port,
        ServiceState::Up,
    )
    .await?;

    let mut sig_term = signal(SignalKind::terminate()).expect("can not setup SIGTERM handler");
    select! {
        _ = ctrl_c() => { log::info!("received Ctrl-C"); }
        _ = sig_term.recv() => { log::info!("received SIGTERM"); }
        result = run_axum_server(settings.http.addr, settings.http.port, transcription.clone()) => { result?; }
    }

    log::debug!("Stop Transcription service");
    shutdown_tx.send(())?;

    let mut tasks = transcription.tasks.lock().await;
    tasks.retain(|task| !task.is_finished());

    if !tasks.is_empty() {
        log::info!("waiting for remaining {} tasks to finish", tasks.len());
        join_all(tasks.iter_mut()).await;
    }
    log::info!("All tasks are finished");

    Ok(())
}

async fn run_axum_server(address: IpAddr, port: u16, transcription: AppState) -> Result<()> {
    let transcription_model_name = transcription.settings.transcription.model.clone();
    let prometheus_state = transcription.prometheus_state.clone();

    // TODO: Add Metrics endpoint
    // TODO: Add bearer token verification
    let app = Router::new()
        .route("/start", post(start_transcription))
        .route("/metrics", get(async move || prometheus_state.metrics()))
        .route(
            "/model",
            get(async move || transcription_model_name.clone()),
        )
        .with_state(transcription);

    let listener = TcpListener::bind((address, port)).await?;

    // Server up and running, ready to process requests
    set_service_state(ServiceState::Ready);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn start_transcription(
    State(transcription): State<AppState>,
    Json(room_info): Json<StartTranscriptionRequestBody>,
) {
    transcription.spawn_session(room_info).await.unwrap();
}

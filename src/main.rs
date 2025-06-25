// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#![allow(clippy::module_name_repetitions)]

use std::{net::IpAddr, sync::Arc};

use anyhow::{Context, Result};
use axum::{
    Router,
    extract::{Path, State},
    routing::post,
};
use futures::future::join_all;
use settings::Settings;
use tokio::{
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
mod settings;
mod transcription;

pub(crate) struct AppState {
    pub(crate) tasks: Arc<Mutex<Vec<JoinHandle<Result<()>>>>>,
    pub(crate) settings: Arc<Settings>,
    pub(crate) shutdown: broadcast::Receiver<()>,
    pub(crate) whisper_context: Arc<WhisperContext>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            tasks: self.tasks.clone(),
            settings: self.settings.clone(),
            shutdown: self.shutdown.resubscribe(),
            whisper_context: self.whisper_context.clone(),
        }
    }
}

// TODO: 4. STT Daten -> OpenTalk Signaling senden
// TODO: 5. Frontend
impl AppState {
    pub(crate) async fn spawn_session(&self, room_id: String) -> Result<()> {
        let context = Arc::new(self.clone());
        log::debug!("Start Transcription session {room_id:?}");
        let mut session = TranscriptionSession::create(context, room_id)
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

    let whisper_context = WhisperContext::new_with_params(
        "ggml-large-v3-turbo-q8_0.bin",
        WhisperContextParameters::default(),
    )?;

    let settings = Arc::new(Settings::load(args.config.as_ref()).context("Failed to read config")?);

    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

    let transcription = AppState {
        tasks: Arc::new(Mutex::new(Vec::new())),
        settings: settings.clone(),
        shutdown: shutdown_rx,
        whisper_context: Arc::new(whisper_context),
    };

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

async fn run_axum_server<A>(address: A, port: u16, transcription: AppState) -> Result<()>
where
    A: Into<IpAddr>,
{
    let app = Router::new()
        .route("/start/{room_id}", post(start_transcription))
        // TODO: Add Metrics endpoint
        // TODO: Add bearer token verification
        .with_state(transcription);
    let listener = tokio::net::TcpListener::bind((address.into(), port)).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

async fn start_transcription(State(transcription): State<AppState>, Path(room_id): Path<String>) {
    transcription.spawn_session(room_id).await.unwrap();
}

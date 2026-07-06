// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};
use axum::{Router, response::IntoResponse, routing::get};
use futures::future::join_all;
use opentalk_orchestrator_client::client::OrchestratorHandle;
use opentalk_transcription_web_api::v1;
use reqwest::StatusCode;
use service_probe::{ServiceState, set_service_state};
use tokio::{net::TcpListener, time::timeout};

use crate::{
    AppState, TranscriptionTasks, metrics::PrometheusState, settings::Settings,
    tasks::ShutdownReceiver,
};

/// Runs the HTTP API server until a shutdown is requested.
///
/// Serves the transcription web API (behind API-key authentication), the Prometheus `/metrics`
/// endpoint and a `/model` endpoint. On shutdown it waits for the remaining transcription tasks
/// to finish, up to a fixed grace period.
pub(crate) async fn run_web_server(
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
        .route(
            "/metrics",
            get(async move || metrics_endpoint(prometheus_state)),
        )
        .route(
            "/model",
            get(async move || transcription_model_name.clone()),
        )
        .with_state(app_state.clone());

    let listener = TcpListener::bind((addr, port))
        .await
        .with_context(|| format!("Failed to bind HTTP server to {addr}:{port}"))?;

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

fn metrics_endpoint(prometheus_state: Arc<PrometheusState>) -> impl IntoResponse {
    match prometheus_state.metrics() {
        Ok(metrics) => (StatusCode::OK, metrics).into_response(),
        Err(err) => {
            log::error!("Failed to get prometheus metrics: {err:?}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

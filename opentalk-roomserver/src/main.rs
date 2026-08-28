// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
// SPDX-FileCopyrightText: Wolfgang Silbermayr <w.silbermayr@opentalk.eu>

//! This crate builds an executable that runs the RoomServer. It implements the
//! [_OpenTalk RoomServer Web API_](opentalk_roomserver_web_api).

use std::{net::ToSocketAddrs as _, path::Path, result, sync::Arc, time::Duration};

use anyhow::Context;
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use clap::Parser;
use cli::{Args, SubCommand};
use futures::TryFutureExt;
use metrics::MetricHandle;
use opentalk_orchestrator_client::{OrchestratorClient, ServiceAddress};
use opentalk_roomserver_common::{
    application_state::ApplicationState,
    settings::{Monitoring, Settings, SettingsFile},
};
use opentalk_roomserver_crypto_provider::ensure_crypto_provider;
use opentalk_roomserver_room::{RoomTaskRegistry, orchestrator::OrchestratorStateProvider};
use service_probe::{ServiceState, start_probe, stop_probe};
use tokio::{
    signal,
    sync::watch::{self, Receiver},
    task::JoinSet,
    time::{Instant, timeout_at},
};

mod api;
mod cli;
mod metrics;
mod tcp_multi_listener;

mod trace;

use tcp_multi_listener::MultiListener;

const SHUTDOWN_GRACE_PERIOD: Duration = Duration::from_secs(42);

pub(crate) async fn wait_shutdown(mut app_state: watch::Receiver<ApplicationState>) {
    let res = app_state.wait_for(ApplicationState::is_shutting_down).await;
    if let Err(e) = res {
        tracing::error!("AppState gone: {e}");
    }
}

pub fn decorate_error(decoration: &'static str) -> impl Fn(anyhow::Error) -> anyhow::Error {
    move |err| err.context(decoration)
}

/// Resolves the socket addresses for the given `address` and `port` and binds a
/// [`MultiListener`] to serve the metrics endpoint on all of them.
async fn bind_metrics_listener(address: &str, port: u16) -> anyhow::Result<MultiListener> {
    let addresses: Vec<_> = (address, port)
        .to_socket_addrs()
        .with_context(|| {
            format!("Failed to determine socket address for http.address: '{address}'")
        })?
        .collect();

    MultiListener::bind(addresses.clone())
        .await
        .with_context(|| {
            format!(
                "Failed to bind metrics to: {}",
                addresses
                    .iter()
                    .map(|addr| format!("{addr}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
}

async fn run_app(config_file_path: Option<&Path>) -> anyhow::Result<()> {
    let (app_state, _) = watch::channel(ApplicationState::Running);
    let settings: Arc<Settings> = Arc::new(SettingsFile::load(config_file_path)?.try_into()?);
    let mut set = JoinSet::new();

    set.spawn(
        shutdown_signal(app_state.subscribe())
            .map_err(decorate_error("Shutdown handler exited with error")),
    );

    trace::init(settings.tracing.as_ref()).context("Failed to initialize tracing")?;

    if settings.controller.is_none() {
        tracing::warn!("Missing controller config, assets and module resources will not be saved")
    }

    if let Some(monitoring) = &settings.monitoring {
        set.spawn(
            start_service_probe(monitoring.clone(), app_state.subscribe())
                .map_err(decorate_error("Service prove exited with error")),
        );
    }

    let mut metric_layer = None;
    if let Some(metric) = &settings.metrics {
        let listener = bind_metrics_listener(settings.http.address.as_ref(), metric.port).await?;

        let (m_layer, metric_handle) = metrics::build_prometheus_layer();
        set.spawn(
            metrics::run_metric_server(
                listener,
                metric.allowlist.clone(),
                metric_handle,
                app_state.subscribe(),
            )
            .map_err(decorate_error("Metric server exited with error")),
        );
        metric_layer = Some(m_layer);
    }

    let addresses = (settings.http.address.as_ref(), settings.http.port)
        .to_socket_addrs()
        .with_context(|| {
            format!(
                "Failed to determine socket address for http.address: '{}'",
                settings.http.address
            )
        })?
        .collect();

    let room_registry = if let Some(orchestrator_config) = &settings.orchestrator {
        let roomserver_key_ids = settings
            .http
            .api_keys
            .inner()
            .iter()
            .map(|key| key.id.clone());

        let (client, handle) =
            OrchestratorClient::create(orchestrator_config.clone(), roomserver_key_ids).await;

        let room_registry = RoomTaskRegistry::new(Some(handle));

        let state_provider = OrchestratorStateProvider::new(room_registry.clone());

        let service_address = match settings.http.service_url.clone() {
            Some(url) => ServiceAddress::Url(url),
            None => ServiceAddress::Port(settings.http.port),
        };

        set.spawn(client.run(
            service_address,
            state_provider,
            wait_shutdown(app_state.subscribe()),
        ));

        room_registry
    } else {
        RoomTaskRegistry::new(None)
    };

    set.spawn(
        api::run_web_server(
            settings,
            addresses,
            room_registry,
            app_state.clone(),
            metric_layer,
        )
        .map_err(decorate_error("API server exited with error")),
    );

    match set.join_next().await {
        // No task was started, this should not happen
        None => tracing::error!("Failed to start any task!"),
        // Task panicked
        Some(Err(e)) => tracing::error!("Task panicked: {e:?}"),
        // Task finished successfully
        Some(Ok(Ok(()))) => {}
        // Task returned an error
        Some(Ok(Err(e))) => tracing::error!("{e:?}"),
    }
    let result = graceful_shutdown(app_state, &mut set).await;
    if result.is_err() {
        set.abort_all();
        return result.context("Forced Shutdown");
    }

    Ok(())
}

impl MetricHandle for PrometheusHandle {
    fn render(&self) -> String {
        self.render()
    }
}

async fn graceful_shutdown(
    app_state: watch::Sender<ApplicationState>,
    set: &mut JoinSet<result::Result<(), anyhow::Error>>,
) -> anyhow::Result<()> {
    tracing::debug!("Starting graceful shutdown");
    app_state.send_replace(ApplicationState::ShuttingDown);
    loop {
        let result = timeout_at(Instant::now() + SHUTDOWN_GRACE_PERIOD, set.join_next()).await;
        match result {
            // Timeout elapsed
            Err(_) => {
                tracing::error!("Not all tasks exited in time!");
                return Err(anyhow::anyhow!("Not all tasks exited in time!"));
            }
            // All tasks shut down
            Ok(None) => return Ok(()),
            // Task exited successfully
            Ok(Some(Ok(Ok(())))) => tracing::info!("Task exited"),
            // Task returned error
            Ok(Some(Ok(Err(e)))) => tracing::error!("Task error: {e:?}"),
            // Task panicked
            Ok(Some(Err(e))) => tracing::error!("Task Panic: {e:?}"),
        }
    }
}

async fn shutdown_signal(app_state: Receiver<ApplicationState>) -> anyhow::Result<()> {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
        () = wait_shutdown(app_state) => {},
    }

    Ok(())
}

async fn start_service_probe(
    monitoring: Monitoring,
    mut app_state_receiver: Receiver<ApplicationState>,
) -> Result<(), anyhow::Error> {
    start_probe(monitoring.addr, monitoring.port, ServiceState::Up)
        .await
        .context("Failed to start monitoring endpoint")?;
    app_state_receiver
        .wait_for(ApplicationState::is_shutting_down)
        .await?;
    stop_probe().await;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    ensure_crypto_provider();

    let args = Args::parse();

    if args.run_tasks().should_exit() {
        return Ok(());
    }

    match args.cmd {
        Some(SubCommand::Openapi(command)) => {
            cli::openapi::handle_command(command)?;
        }
        Some(SubCommand::Health(command)) => {
            cli::health::handle_command(command, args.config.as_deref()).await?;
        }
        Some(SubCommand::Modules(command)) => {
            cli::modules::handle_command(command);
        }
        None => run_app(args.config.as_deref()).await?,
    }

    Ok(())
}

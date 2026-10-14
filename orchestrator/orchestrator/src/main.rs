// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{net::SocketAddr, sync::Arc};

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::{ConnectInfo, MatchedPath, OriginalUri, Request, State, WebSocketUpgrade},
    http::StatusCode,
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Response},
    routing::{any, get},
};
use cidr::IpInet;
use clap::Parser;
use opentalk_roomserver_web_api::v1::rooms;
use opentalk_service_auth::{
    ApiKey, ApiKeyId,
    service::{ApiKeyAuthorization, ApiKeys},
};
use opentalk_types_api_common::error::ApiError;
use reqwest::{Client, header::AUTHORIZATION};
use service_probe::{ServiceState, start_probe, stop_probe};
use tokio::{
    select,
    signal::{
        self,
        unix::{SignalKind, signal},
    },
};
use tower_http::trace::TraceLayer;
use url::Url;

use crate::{
    service_instance::registration::handle_socket,
    settings::{Settings, monitoring::Monitoring, storage::Storage},
    storage::{OrchestratorStorage, local::LocalStorage, redis::RedisStorage},
    tasks::{ShutdownReceiver, Tasks},
};

mod cli;
mod livekit;
mod logging;
mod proxy;
mod recorder;
mod roomserver;
mod service_instance;
mod settings;
mod storage;
mod tasks;
mod transcription;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ServiceType {
    Recorder,
    Roomserver,
    Transcription,
}

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    client: Client,
    public_url: Url,
    service_keys: ApiKeys,
    storage: Arc<dyn OrchestratorStorage>,
}

impl AppState {
    fn new(storage: Arc<dyn OrchestratorStorage>, service_keys: ApiKeys, public_url: Url) -> Self {
        Self {
            client: Default::default(),
            public_url,
            service_keys,
            storage,
        }
    }

    /// Returns true if any of the provided service key ids is known to the orchestrator
    fn knows_any_of(&self, service_key_ids: &[ApiKeyId]) -> bool {
        for key_id in service_key_ids {
            let is_known = self
                .service_keys
                .inner()
                .iter()
                .any(|cred| &cred.id == key_id);

            if is_known {
                return true;
            }
        }

        false
    }

    /// Returns the first viable API key
    fn get_api_key_for_key_ids(&self, service_key_ids: &[ApiKeyId]) -> Option<ApiKey> {
        for key_id in service_key_ids {
            if let Some(key) = self
                .service_keys
                .inner()
                .iter()
                .find(|cred| &cred.id == key_id)
            {
                return Some(key.clone());
            }
        }

        None
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Capture logs emitted before initializing the logger from the settings.
    let interim_logger = logging::bootstrap();

    ensure_crypto_provider();
    let args = cli::Args::parse();

    if let Some(cmd) = args.cmd {
        cli::handle_subcommand(cmd, args.config).await?;

        return Ok(());
    }

    let settings = Settings::load(args.config.as_deref())?;

    let tracer_provider = logging::init_logging(interim_logger, settings.logging.as_ref())?;

    tracing::trace!("Tracing enabled");

    let mut tasks = Tasks::new();

    tasks.spawn("shutdown_signal_handler", |shutdown| async move {
        shutdown_signal_handler(shutdown).await;
        Ok(())
    });

    let storage = setup_storage(&mut tasks, settings.clone()).await?;

    let settings_clone = settings.clone();
    tasks.spawn("webserver", |shutdown| {
        run_webserver(settings_clone, storage, shutdown)
    });

    match settings.monitoring {
        Some(monitoring) => {
            tasks.spawn("service_probe", |shutdown| {
                start_service_probe(monitoring, shutdown)
            });
        }
        None => {
            tracing::info!(
                "Monitoring is not configured, not serving /health, /ready or /startup endpoints"
            );
        }
    }

    tasks.wait_for_shutdown().await?;

    opentalk_tracing::shutdown(tracer_provider);

    Ok(())
}

async fn run_webserver(
    settings: Settings,
    storage: Arc<dyn OrchestratorStorage>,
    mut shutdown: ShutdownReceiver,
) -> Result<()> {
    let auth_middleware = settings.http.api_keys.auth_middleware()?;
    let metrics_router = metrics_router(&settings, &auth_middleware).await;

    let state = AppState::new(storage, settings.services.keys, settings.http.public_url);

    let protected = Router::new()
        .route("/register", any(register))
        .nest("/roomserver/v1", rooms::routes())
        .nest("/recording", opentalk_recorder_web_api::v1::routes())
        .nest(
            "/transcription",
            opentalk_transcription_web_api::v1::routes(),
        )
        .layer(auth_middleware)
        .merge(metrics_router);

    let public = Router::new()
        .nest("/roomserver/v1", roomserver::signaling::routes())
        .nest("/livekit", livekit::routes());

    let app = protected
        .merge(public)
        .with_state(state)
        .fallback(not_found_handler)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);

                    let span = tracing::info_span!(
                        "http_request",
                        http.request.method = ?request.method(),
                        http.route = matched_path,
                        http.status_code = tracing::field::Empty,
                        otel.kind = "server",
                    );
                    opentalk_tracing::propagation::set_parent_from_headers(
                        &span,
                        request.headers(),
                    );
                    span
                })
                .on_response(|response: &Response<_>, _duration, span: &tracing::Span| {
                    span.record(
                        "http.status_code",
                        tracing::field::display(response.status()),
                    );
                }),
        )
        .into_make_service_with_connect_info::<SocketAddr>();

    let address = format!("{}:{}", settings.http.address, settings.http.port);

    tracing::info!("Listening on address {address}");
    let listener = tokio::net::TcpListener::bind(address).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(async move { shutdown.wait_for_shutdown().await })
        .await?;

    Ok(())
}

/// Build the router for the `/metrics` endpoint based on the provided settings
async fn metrics_router(
    settings: &Settings,
    auth_middleware: &ApiKeyAuthorization,
) -> Router<AppState> {
    let router = Router::new().route("/metrics", get(metrics));

    let router = if settings.metrics.disable_token_auth {
        tracing::info!("Disabled token authorization for the /metrics endpoint");
        router
    } else {
        if let Some(token) = &settings.metrics.bearer_token {
            tracing::info!("Using plain bearer token authorization for the /metrics endpoint");
            router.layer(from_fn_with_state(
                Arc::new(token.clone()),
                bearer_middleware,
            ))
        } else {
            tracing::info!(
                "Using JWT orchestrator API key authorization for the /metrics endpoint"
            );
            router.layer(auth_middleware.clone())
        }
    };

    if let Some(allowlist) = &settings.metrics.allowlist {
        if allowlist.is_empty() {
            tracing::warn!(
                "Using empty IP allowlist for the /metrics endpoint, every request will be denied"
            )
        } else {
            tracing::info!("Using IP allowlist for the /metrics endpoint: [{allowlist:?}]");
        }

        router.layer(from_fn_with_state(
            Arc::new(allowlist.clone()),
            allowlist_middleware,
        ))
    } else {
        router
    }
}

/// Middleware that checks that the requesting client IP matches an entry in the allowlist
async fn allowlist_middleware(
    State(allowlist): State<Arc<Vec<IpInet>>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let ip = peer.ip().to_canonical();

    if allowlist.iter().any(|net| net.contains(&ip)) {
        next.run(request).await
    } else {
        tracing::debug!(
            "Access to the metrics endpoint from IP address {ip} was denied. Access allowed from: [{allowlist:?}]."
        );

        ApiError::forbidden()
            .with_code("ip_not_allowed")
            .into_response()
    }
}

/// Middleware that checks for `Bearer <token>` in the AUTHORIZATION header
async fn bearer_middleware(
    State(expected_token): State<Arc<String>>,
    request: Request,
    next: Next,
) -> Response {
    let presented = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().strip_prefix("Bearer "));

    if presented.is_some_and(|token| token == *expected_token) {
        next.run(request).await
    } else {
        ApiError::unauthorized()
            .with_code("invalid_api_token")
            .with_message("invalid or missing bearer token")
            .into_response()
    }
}

/// Sets up the storage backend based on the provided settings
async fn setup_storage(
    tasks: &mut Tasks,
    settings: Settings,
) -> Result<Arc<dyn OrchestratorStorage>> {
    let storage: Arc<dyn OrchestratorStorage> = match settings.storage {
        Storage::Local => {
            tracing::info!("Using local storage for orchestrator state");
            Arc::new(LocalStorage::new())
        }
        Storage::Redis { url } => {
            let storage = RedisStorage::init(tasks, &url)
                .await
                .context("Failed to create Redis storage")?;
            tracing::info!("Connected to redis storage at '{url}'");
            Arc::new(storage)
        }
    };

    Ok(storage)
}

async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    match state.storage.get_orchestrator_metrics().await {
        Ok(metrics) => (StatusCode::OK, Json(metrics)).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch orchestrator metrics: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn register(
    ws: WebSocketUpgrade,
    ConnectInfo(socket_addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> Response {
    ws.protocols(["opentalk-orchestrator-json-v1.0"])
        .on_upgrade(move |socket| handle_socket(socket, socket_addr, state))
}

async fn not_found_handler(OriginalUri(uri): OriginalUri) -> impl IntoResponse {
    tracing::debug!("Received request for unknown route: {}", uri);

    (StatusCode::NOT_FOUND, "requested route was not found")
}

pub async fn start_service_probe(
    monitoring: Monitoring,
    mut shutdown: ShutdownReceiver,
) -> Result<()> {
    start_probe(monitoring.address, monitoring.port, ServiceState::Ready)
        .await
        .context("Failed to start monitotoring endpoint")?;

    shutdown.wait_for_shutdown().await;

    stop_probe().await;
    Ok(())
}

pub async fn shutdown_signal_handler(mut shutdown_signal: ShutdownReceiver) {
    let mut sig_term = signal(SignalKind::terminate()).expect("cannot setup SIGTERM handler");
    select! {
        _ = signal::ctrl_c() => { tracing::debug!("received Ctrl-C"); }
        _ = sig_term.recv() => { tracing::debug!("received SIGTERM"); }
        _ = shutdown_signal.wait_for_shutdown() => {
            tracing::trace!("Shutdown handler received shutdown signal from application state");
            return;
        }
    }

    tracing::info!("Received shutdown signal...");
}

/// `rustls` and `jsonwebtoken` depend on a `CryptoProvider` being configured.
/// If no provider was explicitly configured, a provider will be derived from
/// the enabled features. Since there are many crates that depend on rustls and
/// `jsonwebtoken`, we don't have complete control over the enabled features.
/// If the configuration via feature is ambiguous these crates will panic.
///
/// Here we ensure that these crates are explicitly configured.
fn ensure_crypto_provider() {
    rustls::crypto::CryptoProvider::install_default(rustls::crypto::aws_lc_rs::default_provider())
        .expect("valid default crypto provider expected");

    jsonwebtoken::crypto::CryptoProvider::install_default(
        &jsonwebtoken::crypto::aws_lc::DEFAULT_PROVIDER,
    )
    .expect("valid default crypto provider expected");
}

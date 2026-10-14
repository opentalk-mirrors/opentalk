// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # Service probe
//!
//! This crate provides an easy way to start a HTTP server that can be used for
//! making the status of a service transparent to observers. The main use case is
//! to communicate information about the health status of a service in containerized
//! environments.
//!
//! Tasks and synchronization throughout this crate uses [`tokio`]
//! functionality, so the runtime must be present and running when the functions
//! of this crate are called.
//!
//! To start the service probe, add the following code to your service:
//!
//! ```no_run
//! # async {
//! use service_probe::{start_probe, ServiceState, set_service_state};
//!
//! // The probe server is started in the background. Up signals that the services is starting.
//! start_probe([0u8, 0, 0, 0], 11333, ServiceState::Up).await.unwrap();
//!
//! // If everything is ready, we set the state to ready.
//! set_service_state(ServiceState::Ready);
//!
//! # };
//! ```
#![deny(
    bad_style,
    missing_debug_implementations,
    missing_docs,
    overflowing_literals,
    patterns_in_fns_without_body,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

use std::{convert::Infallible, net::IpAddr, time::Duration};

use http_body_util::Full;
use hyper::{Method, Request, Response, StatusCode, server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use log::{debug, error, info};
pub use service_probe_common::ServiceState;
use snafu::{ResultExt as _, Snafu};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{RwLock, oneshot},
    task::JoinHandle,
};

struct ProbeTaskHandle {
    shutdown_sender: oneshot::Sender<()>,
    join_handle: JoinHandle<()>,
}

static SERVICE_STATE: std::sync::RwLock<ServiceState> = std::sync::RwLock::new(ServiceState::Up);
static PROBE_TASK_HANDLE: RwLock<Option<ProbeTaskHandle>> = RwLock::const_new(None);

/// The grace period given to the probe for shutting itself down.
pub const SHUTDOWN_GRACE_PERIOD: Duration = Duration::from_millis(500);

/// The error that can happen during startup of the service probe.
#[derive(Debug, Snafu)]
pub enum ProbeStartError {
    /// The service probe has been started already.
    AlreadyStarted,

    /// The socket cannot be used for providing the service probe.
    SocketUnavailable {
        /// The source error
        source: std::io::Error,
    },
}

/// Set the state of the service.
///
/// After this function has been called, requests to the probe endpoint will return the new state.
pub fn set_service_state(state: ServiceState) {
    let mut state_lock = SERVICE_STATE
        .write()
        .expect("rwlock poisoning should be impossible with the implemented control flow");
    if state != *state_lock {
        debug!("Service state change: {} to {}.", *state_lock, state);
        *state_lock = state;
    }
}

/// Get the state of the service.
///
/// This is the state that is returned by the probe endpoint.
pub fn get_service_state() -> ServiceState {
    *SERVICE_STATE
        .read()
        .expect("rwlock poisoning should be impossible with the implemented control flow")
}

/// Start the probe HTTP service.
///
/// This opens a HTTP v1 server on the selected address and port which will serve the state in `GET` requests to `/health`.
pub async fn start_probe<A>(
    address: A,
    port: u16,
    initial_state: ServiceState,
) -> Result<(), ProbeStartError>
where
    A: Into<IpAddr>,
{
    let mut probe_task_handle = PROBE_TASK_HANDLE.write().await;

    if probe_task_handle.is_some() {
        return Err(ProbeStartError::AlreadyStarted);
    }

    let (shutdown_sender, shutdown_receiver) = oneshot::channel();

    let ip_address: IpAddr = address.into();

    let listener = TcpListener::bind((ip_address, port))
        .await
        .context(SocketUnavailableSnafu)?;
    info!(
        "Service readiness probe listening on http://{ip_address}:{port}/ with initial state {initial_state}"
    );

    // We set the state after the last possible error. If this function errors, it should have no side effects.
    set_service_state(initial_state);
    let join_handle = tokio::task::spawn(run_probe_server(listener, shutdown_receiver));

    *probe_task_handle = Some(ProbeTaskHandle {
        shutdown_sender,
        join_handle,
    });

    Ok(())
}

/// Stop the probe HTTP service.
///
/// There is a grace period defined as [`SHUTDOWN_GRACE_PERIOD`]. If the
/// grace period is exceeded, no further action will be taken, but an error will
/// be logged and this function returns.
pub async fn stop_probe() {
    let Some(ProbeTaskHandle {
        shutdown_sender,
        join_handle,
    }) = PROBE_TASK_HANDLE.write().await.take()
    else {
        return;
    };

    let _ = shutdown_sender.send(());

    debug!("Shutting down service readiness probe");

    if let Err(_elapsed) = tokio::time::timeout(SHUTDOWN_GRACE_PERIOD, join_handle).await {
        error!("Error shutting down the service readiness probe");
    }
}

async fn run_probe_server(listener: TcpListener, mut shutdown_receiver: oneshot::Receiver<()>) {
    loop {
        tokio::select! {
            accept = listener.accept() => {
                match accept {
                    Ok((stream, _addr)) => {
                        _ = tokio::spawn(handle_accept(stream));
                    }
                    Err(e) => {
                        error!("Error accepting connection for service readiness probe: {e:?}");
                    }
                }
            }
            _ = &mut shutdown_receiver => {
                return;
            }
        }
    }
}

async fn handle_accept(stream: TcpStream) {
    let io = TokioIo::new(stream);
    let service = service_fn(handle_request);

    if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
        error!("Error serving connection for service readiness probe: {e:?}");
    }
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<&'static [u8]>>, Infallible> {
    let (status_code, body) = match *req.method() {
        Method::GET => {
            let path = req.uri().path();
            let state = get_service_state();
            if ["", "/", "/health", "/health/"].contains(&path) {
                (StatusCode::OK, state.as_str())
            } else if ["/ready", "/ready/"].contains(&path) {
                if state == ServiceState::Ready {
                    (StatusCode::OK, state.as_str())
                } else {
                    (StatusCode::SERVICE_UNAVAILABLE, state.as_str())
                }
            } else {
                (StatusCode::NOT_FOUND, "Not found")
            }
        }
        Method::HEAD => (StatusCode::OK, ""),
        _ => (StatusCode::METHOD_NOT_ALLOWED, "Method not allowed"),
    };
    let mut response = Response::new(Full::new(body.as_bytes()));
    *response.status_mut() = status_code;
    Ok(response)
}

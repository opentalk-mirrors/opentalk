// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use anyhow::{Chain, Context, Result};
use async_trait::async_trait;
use opentalk_orchestrator_shared::{
    Event, Metrics, Register, RegisterData, RegisterResponse, RegisterType, ServiceAddress,
    error::RegistrationError, services::ServiceResource,
};
use opentalk_service_auth::ApiKeyId;
use tokio::{sync::mpsc, time::Instant};
pub use url::Url;

use crate::{
    config::OrchestratorConfig,
    signaling_socket::{BuildWsRequestError, SignalingSocket},
};

const METRIC_INTERVAL: Duration = Duration::from_secs(1);
const RECONNECT_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Failed to build initial websocket request")]
    BuildWsRequest(#[from] BuildWsRequestError),

    #[error("Failed to resolve resource conflicts")]
    UnresolvedResourceConflict,

    #[error(transparent)]
    Recoverable(#[from] anyhow::Error),
}

#[async_trait]
pub trait StateProvider {
    /// Return the type of service that is registering at the orchestrator
    async fn register_type(&mut self) -> RegisterType;
    /// Return the current metrics of the service
    async fn metrics(&mut self) -> Metrics;
    /// Called when the orchestrator reports that a resource collision has occurred
    ///
    /// The service must clear the resources to register at the orchestrator. Returning Ok(())
    /// will cause the client to immediately register at the orchestrator again.
    async fn on_resource_collision(&mut self, resources: &[ServiceResource]) -> anyhow::Result<()>;
}

/// The client to connect to the orchestrator
pub struct OrchestratorClient {
    /// List of key ids that can be used to authorize requests to implementing service
    key_ids: Vec<ApiKeyId>,
    /// The orchestrator configuration
    config: OrchestratorConfig,
    /// Internal receiver for events that shall be forwarded to the orchestrator
    event_receiver: mpsc::Receiver<Event>,
}

/// Handle to send events to the [`OrchestratorClient`] task
#[derive(Debug, Clone)]
pub struct OrchestratorHandle(pub mpsc::Sender<Event>);

impl OrchestratorHandle {
    pub async fn send_event<E: Into<Event> + Send + 'static>(
        &self,
        event: E,
    ) -> anyhow::Result<()> {
        self.0
            .send(event.into())
            .await
            .map_err(|_| anyhow::anyhow!("Orchestrator client exited"))
    }
}

impl OrchestratorClient {
    /// Creates a new [`OrchestratorClient`]
    pub async fn create<K: IntoIterator<Item = ApiKeyId>>(
        config: OrchestratorConfig,
        key_ids: K,
    ) -> (Self, OrchestratorHandle) {
        // Client -> Orchestrator
        let (event_sender, event_receiver) = mpsc::channel::<Event>(32);

        (
            Self {
                key_ids: key_ids.into_iter().collect(),
                config,
                event_receiver,
            },
            OrchestratorHandle(event_sender),
        )
    }

    /// Connect to the given orchestrator address and continuously synchronize service metrics
    ///
    /// Once connected, the client task will send an initial dump of available metrics. While
    /// connected, the metrics are sent in a fixed interval.
    ///
    /// The client will indefinitely attempt to reconnect to the orchestrator when a recoverable
    /// error is encountered (E.g. network failure).
    ///
    /// Returns with [Ok] when the [`OrchestratorHandle`] is dropped or the shutdown_signal is
    /// received. Returns with [Err] when the encountered error is deemed to be non-recoverable
    /// (E.g. configuration error).
    pub async fn run<P>(
        mut self,
        client_address: ServiceAddress,
        mut state_provider: P,
        shutdown_signal: impl Future<Output = ()>,
    ) -> Result<(), anyhow::Error>
    where
        P: StateProvider + Send + 'static,
    {
        tokio::pin!(shutdown_signal);

        log::info!("Connecting to orchestrator at {} ...", self.config.url);
        loop {
            let Err(error) = self
                .inner_run(
                    client_address.clone(),
                    &mut state_provider,
                    &mut shutdown_signal,
                )
                .await
            else {
                // Gracefully exiting
                return Ok(());
            };

            match error {
                ClientError::Recoverable(recoverable_error) => {
                    log::warn!(
                        "{recoverable_error}, retrying in {RECONNECT_INTERVAL:?}\n{}",
                        ErrorCauses::from(&recoverable_error),
                    );

                    tokio::select! {
                        () = tokio::time::sleep(RECONNECT_INTERVAL) => {
                            continue;
                        },
                        () =  &mut shutdown_signal => {
                            return Ok(());
                        }
                    }
                }
                critical => {
                    return Err(critical.into());
                }
            }
        }
    }

    async fn inner_run<P>(
        &mut self,
        client_address: ServiceAddress,
        state_provider: &mut P,
        shutdown_signal: impl Future<Output = ()>,
    ) -> Result<(), ClientError>
    where
        P: StateProvider + Send + 'static,
    {
        tokio::pin!(shutdown_signal);

        // Clear the channel to avoid pushing outdated events when reconnected
        loop {
            match self.event_receiver.try_recv() {
                Ok(_) => continue,
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    log::debug!("client handle was dropped");
                    return Ok(());
                }
                Err(mpsc::error::TryRecvError::Empty) => break,
            }
        }

        let mut socket = tokio::select! {
            socket = self.connect_and_register(client_address, state_provider) => {
                socket?
            }
            () = &mut shutdown_signal => {
                log::debug!("received shutdown signal");
                return Ok(())
            }
        };

        log::info!("Connected to orchestrator");

        let result = self
            .event_loop(state_provider, &mut socket, &mut shutdown_signal)
            .await;

        socket.close().await;

        Ok(result?)
    }

    /// Continuously send client metrics and events to the orchestrator
    async fn event_loop<P>(
        &mut self,
        state_provider: &mut P,
        socket: &mut SignalingSocket,
        shutdown_signal: impl Future<Output = ()>,
    ) -> Result<()>
    where
        P: StateProvider + Send + 'static,
    {
        let mut interval = tokio::time::interval_at(Instant::now(), METRIC_INTERVAL);
        tokio::pin!(shutdown_signal);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    socket.send(Event::Metrics(state_provider.metrics().await)).await?;
                }
                data = self.event_receiver.recv() => {
                    match data {
                        Some(event) => {
                            socket.send::<Event>(event).await?;
                        },
                        None => {
                            log::debug!("client handle was dropped");
                            return Ok(())
                        },
                    }
                }
                () = &mut shutdown_signal => {
                    log::debug!("received shutdown signal");
                    return Ok(())
                }
                result = socket.recv() => {
                    let _: () = result?;
                }
            }
        }
    }

    async fn connect_and_register<P>(
        &self,
        service_address: ServiceAddress,
        state_provider: &mut P,
    ) -> Result<SignalingSocket, ClientError>
    where
        P: StateProvider + Send + 'static,
    {
        let mut socket = SignalingSocket::connect(&self.config).await?;

        self.register(&mut socket, service_address, state_provider)
            .await?;

        Ok(socket)
    }

    /// Send a registration message to the orchestrator and wait for its response
    async fn register<P>(
        &self,
        socket: &mut SignalingSocket,
        service_address: ServiceAddress,
        state_provider: &mut P,
    ) -> Result<(), ClientError>
    where
        P: StateProvider + Send + 'static,
    {
        const MAX_RETRIES: u32 = 3;
        let mut retries = 0;

        loop {
            let registration = Register {
                register_data: RegisterData {
                    service_address: service_address.clone(),
                    api_key_ids: self.key_ids.clone(),
                    metrics: state_provider.metrics().await,
                },
                register_type: state_provider.register_type().await,
            };

            socket.send(registration).await?;

            let registration_response = socket.recv::<RegisterResponse>().await?;

            match registration_response {
                RegisterResponse::Success => return Ok(()),
                RegisterResponse::Error(RegistrationError::ResourceConflict(resources)) => {
                    if retries >= MAX_RETRIES {
                        return Err(ClientError::UnresolvedResourceConflict);
                    }
                    retries += 1;

                    state_provider
                        .on_resource_collision(&resources)
                        .await
                        .context("Failed to handle resource collision")?;
                }
                RegisterResponse::Error(err) => {
                    return Err(err).context("Failed to register at orchestrator")?;
                }
            }
        }
    }
}

/// Formatting helper for the anyhow error chain
struct ErrorCauses<'a>(Chain<'a>);

impl<'a> From<&'a anyhow::Error> for ErrorCauses<'a> {
    fn from(chain: &'a anyhow::Error) -> ErrorCauses<'a> {
        Self(chain.chain())
    }
}

impl std::fmt::Display for ErrorCauses<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Caused by:")?;

        // skip the original error
        let causes = self.0.clone().skip(1);

        for (n, cause) in causes.enumerate() {
            f.write_fmt(format_args!("\n{:>3}: ", n))?;
            std::fmt::Display::fmt(cause, f)?;
        }

        Ok(())
    }
}

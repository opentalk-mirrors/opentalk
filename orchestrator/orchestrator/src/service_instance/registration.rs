// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{net::SocketAddr, time::Duration};

use anyhow::{Context, Result};
use axum::extract::ws::{CloseFrame, Message, WebSocket, close_code};
use opentalk_orchestrator_shared::{
    Metrics, Register, RegisterResponse, RegisterType, ServiceAddress, ServiceKind,
    error::RegistrationError,
};
use opentalk_service_auth::ApiKeyId;
use tokio::time::timeout;
use url::Url;

use crate::{AppState, service_instance::runner::InstanceRunner, storage::AddInstanceError};

pub(crate) const REGISTRATION_TIMEOUT: Duration = Duration::from_secs(10);

/// Number of additional registration attempts granted to a service instance that reported
/// resolved resource conflicts
pub(crate) const MAX_RESOURCE_CONFLICT_RETRIES: u32 = 3;

/// Local error type for the registration process
#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Registration(#[from] RegistrationError),
    #[error("connection dropped by client")]
    ConnectionDropped,
    #[error("websocket error {0}")]
    WebSocket(#[from] axum::Error),
}

pub(crate) async fn handle_socket(mut socket: WebSocket, socket_addr: SocketAddr, state: AppState) {
    let registration = match state.register_service(&mut socket, socket_addr).await {
        Ok(registration) => registration,
        Err(err) => {
            tracing::error!("failed to register new service: {err}");

            if let Error::Registration(registration_error) = err {
                send_registration_response(&mut socket, registration_error).await;
            }

            close_socket(socket, close_code::NORMAL, "registration failed").await;

            return;
        }
    };

    let address = registration.address;
    let service_kind = ServiceKind::from(&registration.register_type);

    let runner = InstanceRunner::new(socket, address.clone(), service_kind, state.storage.clone());

    match runner.run().await {
        Ok(()) => {
            tracing::info!(
                "disconnected from {service_kind} ({address}), connection closed by service"
            );
        }
        Err(err) => {
            tracing::error!("unexpected disconnect from {service_kind} ({address}): {err:?}");
        }
    }
}

/// Receive and parse the [`Register`] message on the given socket
async fn receive_registration_message(socket: &mut WebSocket) -> Result<Register, Error> {
    let Ok(message) = timeout(REGISTRATION_TIMEOUT, socket.recv()).await else {
        tracing::error!("did not receive registration message for {REGISTRATION_TIMEOUT:?}");
        return Err(RegistrationError::Timeout.into());
    };

    let Some(message) = message.transpose()? else {
        return Err(Error::ConnectionDropped);
    };

    let parse_message: Result<Register, _> = match message {
        Message::Text(utf8_bytes) => serde_json::from_str(&utf8_bytes),
        Message::Binary(bytes) => serde_json::from_slice(&bytes),
        _ => return Err(RegistrationError::InvalidMessageType.into()),
    };

    match parse_message.context("unable to parse register message") {
        Ok(message) => Ok(message),
        Err(err) => {
            tracing::debug!("failed to parse registration message: {err}");
            Err(RegistrationError::InvalidJson.into())
        }
    }
}

/// Wait for the registration request on the given socket
async fn receive_registration(
    socket: &mut WebSocket,
    socket_addr: SocketAddr,
) -> Result<ServiceRegistration, Error> {
    let Register {
        register_data,
        register_type,
    } = receive_registration_message(socket).await?;

    let address = resolve_service_address(register_data.service_address, socket_addr)?;

    Ok(ServiceRegistration {
        address,
        register_type,
        api_key_ids: register_data.api_key_ids,
        metrics: register_data.metrics,
    })
}

/// Determine the address on which the registering service instance can be reached
///
/// Instances that only provided a port are addressed on the IP address they connected from.
fn resolve_service_address(
    service_address: ServiceAddress,
    socket_addr: SocketAddr,
) -> Result<Url, RegistrationError> {
    match service_address {
        ServiceAddress::Url(url) => Ok(url),
        ServiceAddress::Port(port) => Url::parse(&format!("http://{}:{port}", socket_addr.ip()))
            .map_err(|err| {
                tracing::error!(
                    "failed to build service address from socket address and port: {err}"
                );
                RegistrationError::InvalidServiceAddress
            }),
    }
}

#[derive(Debug, Clone)]
/// The data received from a service instance during registration
pub struct ServiceRegistration {
    /// The address of the service instance
    pub address: Url,
    /// The type of the service instance
    pub register_type: RegisterType,
    /// The api key ids provided by the service instance
    pub api_key_ids: Vec<ApiKeyId>,
    /// The initial metrics provided by the service instance
    pub metrics: Metrics,
}

impl AppState {
    /// Register a service instance on the given socket
    ///
    /// A [`RegistrationError::ResourceConflict`] is reported to the service instance without
    /// closing the connection, because the instance is expected to release the conflicting
    /// resources and then register again on the same connection. Keeping the connection open
    /// avoids a reconnect, during which the instance could collide with yet another instance.
    ///
    /// The connection is only given up once the instance ran out of retries, or reported a
    /// conflict-unrelated error.
    async fn register_service(
        &self,
        socket: &mut WebSocket,
        socket_addr: SocketAddr,
    ) -> Result<ServiceRegistration, Error> {
        let mut retries_left = MAX_RESOURCE_CONFLICT_RETRIES;

        loop {
            let registration = receive_registration(socket, socket_addr).await?;
            let service_kind = ServiceKind::from(&registration.register_type);

            tracing::debug!(
                "Received {service_kind} registration request from {}",
                registration.address
            );

            match self.register_instance(registration.clone()).await {
                Ok(()) => {
                    send_registration_response(socket, RegisterResponse::Success).await;

                    return Ok(registration);
                }
                Err(RegistrationError::ResourceConflict(resources)) if retries_left > 0 => {
                    retries_left -= 1;

                    tracing::info!(
                        "{service_kind} ({}) has {} resource(s) managed by other instances, \
                         awaiting re-registration ({retries_left} retries left)",
                        registration.address,
                        resources.len(),
                    );

                    send_registration_response(
                        socket,
                        RegistrationError::ResourceConflict(resources),
                    )
                    .await;
                }
                Err(registration_error) => {
                    tracing::error!(
                        "failed to register {service_kind} ({}): {registration_error}",
                        registration.address
                    );

                    return Err(registration_error.into());
                }
            }
        }
    }

    /// Try to register a new service instance
    async fn register_instance(
        &self,
        registration: ServiceRegistration,
    ) -> Result<(), RegistrationError> {
        if !self.knows_any_of(&registration.api_key_ids) {
            return Err(RegistrationError::UnknownApiKeyIds);
        }

        if let Err(e) = self.storage.add_instance(registration).await {
            return match e {
                AddInstanceError::RegistrationError(registration_error) => Err(registration_error),
                AddInstanceError::Internal(error) => {
                    tracing::error!("Failed to register service: {error}");
                    Err(RegistrationError::Internal)
                }
            };
        }
        Ok(())
    }
}

async fn send_registration_response(socket: &mut WebSocket, response: impl Into<RegisterResponse>) {
    let response = response.into();

    match serde_json::to_string(&response) {
        Ok(error_message) => {
            if let Err(err) = socket.send(Message::Text(error_message.into())).await {
                tracing::error!("failed to send registration response: {err} ")
            }
        }
        Err(err) => {
            tracing::error!("failed to serialize registration response: {err}");
        }
    };
}

async fn close_socket<S: AsRef<str>>(mut socket: WebSocket, code: u16, reason: S) {
    if let Err(err) = socket
        .send(Message::Close(Some(CloseFrame {
            code,
            reason: reason.as_ref().into(),
        })))
        .await
    {
        tracing::debug!("Failed to close websocket connection: {err}");
    };
}

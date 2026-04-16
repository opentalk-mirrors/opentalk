// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{borrow::Cow, net::SocketAddr, time::Duration};

use anyhow::{Context, Result, bail};
use service_probe::{ServiceState, set_service_state};
use sip_auth::{
    ClientAuthenticator, DigestAuthenticator, DigestCredentials, DigestUser, RequestParts,
    ResponseParts,
};
use sip_core::{
    Endpoint,
    transport::{TargetTransportInfo, TransportState},
};
use sip_types::{
    StatusCode,
    header::typed::Contact,
    print::AppendCtx,
    uri::{NameAddr, SipUri},
};
use sip_ua::register::Registration;
use tokio::{
    sync::{broadcast, watch},
    time::{Interval, interval, sleep},
};

use crate::settings::SipRegistrarSettings;

/// How long should the binding stay active
const REGISTRATION_EXPIRY: Duration = Duration::from_secs(300);

/// If the REGISTER fails, wait in between attempts
const INITIAL_RETRY_DELTA: Duration = Duration::from_secs(1);
const MAXIMUM_RETRY_DELTA: Duration = Duration::from_secs(16);

pub(super) struct RegistrationTask {
    endpoint: Endpoint,

    target_uri: SipUri,

    registration: Registration,
    target: TargetTransportInfo,
    authenticator: DigestAuthenticator,

    retry_delta: Duration,

    keep_alive_interval: Interval,

    transport_state: watch::Receiver<TransportState>,

    shutdown: broadcast::Receiver<()>,

    run: bool,
}

impl RegistrationTask {
    pub(super) async fn register_and_spawn(
        endpoint: Endpoint,
        pub_addr: SocketAddr,
        id: SipUri,
        contact: SipUri,
        username: Option<String>,
        settings: SipRegistrarSettings,
        shutdown: broadcast::Receiver<()>,
    ) -> Result<()> {
        let registrar_uri: SipUri = maybe_add_sip_scheme(&settings.registrar)
            .parse()
            .context("Failed to parse registrar SIP uri")?;

        // Check if we send the request to an outbound proxy or directly to the registrar uri
        let target_uri = if let Some(outbound_proxy) = settings.outbound_proxy {
            maybe_add_sip_scheme(&outbound_proxy)
                .parse()
                .context("Failed to parse outbound_proxy SIP uri")?
        } else {
            registrar_uri.clone()
        };

        let registration = Registration::new(
            NameAddr::uri(id),
            Contact::new(NameAddr::uri(contact)),
            registrar_uri,
            REGISTRATION_EXPIRY,
        );

        let mut credentials = DigestCredentials::new();

        // Add username & password credentials if both are set
        if let Some((username, password)) = username.zip(settings.password) {
            let credential = DigestUser::new(username, password);

            if let Some(realm) = settings.realm {
                credentials.add_for_realm(realm, credential);
            } else {
                credentials.set_default(credential);
            }
        }

        let mut authenticator = DigestAuthenticator::new(credentials);
        authenticator.enforce_qop = settings.enforce_qop;

        // Select a transport for the URI to which to send the REGISTER to
        let (transport, target_addr) =
            endpoint
                .select_transport(&target_uri)
                .await
                .with_context(|| {
                    format!(
                        "Failed to select transport for '{}'",
                        target_uri.default_print_ctx()
                    )
                })?;

        let target = TargetTransportInfo {
            via_host_port: Some(pub_addr.into()),
            transport: Some((transport.clone(), target_addr)),
        };

        let mut task = RegistrationTask {
            endpoint,
            target_uri,
            registration,
            target,
            authenticator,
            retry_delta: INITIAL_RETRY_DELTA,
            keep_alive_interval: interval(settings.nat_ping_delta),
            transport_state: transport.watch_state(),
            shutdown,
            run: true,
        };

        task.register(false)
            .await
            .context("Failed to register with the configured registrar")?;

        tokio::spawn(task.run());

        Ok(())
    }

    /// Send a register request and handle authentication using the given session and credentials
    async fn register(&mut self, remove_binding: bool) -> Result<()> {
        for _ in 0..5 {
            let mut request = self.registration.create_register(remove_binding);
            request.headers.insert_named(self.endpoint.allowed());

            self.authenticator.authorize_request(&mut request.headers);

            let mut transaction = self
                .endpoint
                .send_request(request, &mut self.target)
                .await?;

            let response = transaction.receive_final().await?;

            let response_code = response.line.code;

            match response_code.into_u16() {
                200..=299 => {
                    if !remove_binding {
                        self.registration.receive_success_response(response);
                        set_service_state(ServiceState::Ready);
                    } else {
                        set_service_state(ServiceState::Up);
                    }
                    return Ok(());
                }
                401 | 407 => self.authenticator.handle_rejection(
                    RequestParts {
                        line: &transaction.request().msg.line,
                        headers: &transaction.request().msg.headers,
                        body: &transaction.request().msg.body,
                    },
                    ResponseParts {
                        line: &response.line,
                        headers: &response.headers,
                        body: &response.body,
                    },
                )?,
                400..=499 if !remove_binding => {
                    if !self.registration.receive_error_response(response) {
                        bail!(RegisterRejected(response_code));
                    }
                }
                _ => bail!(RegisterRejected(response_code)),
            }
        }

        bail!("Exceeded maximum authentication attempts")
    }

    async fn run(mut self) {
        enum Event {
            BindingExpired,
            TransportClosed,
            SendKeepAlivePing,
            Shutdown,
        }

        while self.run {
            let event = tokio::select! {
                () = self.registration.wait_for_expiry() => {
                    Event::BindingExpired
                }
                _closed = self.transport_state.wait_for(|state| matches!(state, TransportState::Closed(..))) => {
                    Event::TransportClosed
                }
                _ = self.keep_alive_interval.tick() => {
                    Event::SendKeepAlivePing
                }
                _ = self.shutdown.recv() => {
                    Event::Shutdown
                }
            };

            match event {
                Event::BindingExpired => {
                    // Refresh binding
                    if let Err(e) = self.register(false).await {
                        set_service_state(ServiceState::Up);

                        log::warn!("Failed to refresh binding {e:?}");

                        self.sleep_before_retry().await;

                        if e.is::<RegisterRejected>() {
                            // Force registration attempt immediately
                            self.registration.reset_expiry_timer();
                        } else {
                            // If the REGISTER request was not rejected, but another failure happened, reselect transport
                            self.reselect_transport().await;
                        }
                    } else {
                        // Register succeeded, reset retry delta & update service state
                        self.retry_delta = INITIAL_RETRY_DELTA;
                        set_service_state(ServiceState::Ready);
                    }
                }
                Event::TransportClosed => {
                    set_service_state(ServiceState::Up);
                    self.reselect_transport().await;
                }
                Event::SendKeepAlivePing => {
                    if let Some((transport, target)) = &self.target.transport
                        && transport.is_udp()
                    {
                        // Only send pings to the target if a UDP transport is used, for connection this is not needed
                        if let Err(e) = transport.send(b"\r\n", *target).await {
                            log::error!("failed to send keep-alive, {e}");
                            set_service_state(ServiceState::Up);
                            self.reselect_transport().await;
                        }
                    }
                }
                Event::Shutdown => {
                    // Remove binding and exit task
                    if let Err(e) = self.register(true).await {
                        set_service_state(ServiceState::Up);
                        log::error!("Failed to remove binding, {e:?}");
                    }

                    return;
                }
            }
        }
    }

    async fn sleep_before_retry(&mut self) {
        log::info!(
            "Waiting {:?} before re-trying to register",
            self.retry_delta
        );

        sleep(self.retry_delta).await;

        tokio::select! {
            _ = sleep(self.retry_delta) => {
                self.retry_delta = (self.retry_delta * 2).min(MAXIMUM_RETRY_DELTA);
            }
            _ = self.shutdown.recv() => {
                self.run = false;
            }
        }
    }

    // Reselect transport & destination for target registrar
    async fn reselect_transport(&mut self) {
        while self.run {
            match self.endpoint.select_transport(&self.target_uri).await {
                Ok((transport, target_addr)) => {
                    self.transport_state = transport.watch_state();
                    self.target.transport = Some((transport, target_addr));

                    // Force registration attempt immediately, since a new transport is being used
                    self.registration.reset_expiry_timer();
                }
                Err(err) => {
                    log::warn!(
                        "Failed to select transport for '{:?}': {err}, waiting {:?} seconds before retrying",
                        self.target_uri,
                        self.retry_delta,
                    );

                    tokio::select! {
                        _ = sleep(self.retry_delta) => {
                            self.retry_delta = (self.retry_delta * 2).min(MAXIMUM_RETRY_DELTA);
                            continue;
                        }
                        _ = self.shutdown.recv() => {
                            self.run = false;
                            return;
                        }
                    }
                }
            };

            return;
        }
    }
}

fn maybe_add_sip_scheme(i: &str) -> Cow<'_, str> {
    if i.starts_with("sip:") || i.starts_with("sips:") {
        Cow::Borrowed(i)
    } else {
        Cow::Owned(format!("sip:{i}"))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Registration attempt rejected")]
struct RegisterRejected(StatusCode);

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::http::HttpClient;
use crate::media::{MediaPipeline, Track};
use crate::settings::{DisplayNameSource, MonitoringSettings, Settings, SipRegistrarSettings};
use crate::signaling::Signaling;
use anyhow::{Context, Result, bail};
use bytesstr::BytesStr;
use service_probe::{ServiceState, set_service_state, start_probe};
use sip_auth::{ClientAuthenticator, DigestUser, RequestParts, ResponseParts};
use sip_auth::{DigestAuthenticator, DigestCredentials};
use sip_core::transport::streaming::StreamingListenerBuilder;
use sip_core::transport::tcp::{TcpConnector, TcpListener};
use sip_core::transport::udp::Udp;
use sip_core::transport::{TargetTransportInfo, TpHandle};
use sip_core::{Endpoint, EndpointBuilder, IncomingRequest, Layer, MayTake};
use sip_types::header::typed::{Contact, ContentType, FromTo, Refresher};
use sip_types::print::AppendCtx;
use sip_types::uri::{NameAddr, SipUri, SipUriUserPart};
use sip_types::{Method, Name, StatusCode};
use sip_ua::dialog::{Dialog, DialogLayer};
use sip_ua::invite::InviteLayer;
use sip_ua::invite::acceptor::InviteAcceptor;
use sip_ua::register::Registration;
use std::borrow::Cow;
use std::net::{IpAddr, SocketAddr};
use std::pin::pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::lookup_host;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::broadcast;
use tokio::time::{interval, sleep};

mod codec;
mod media_control;
mod options;
mod sdp;
mod update;

pub use codec::{H264PacketizationMode, SdpRecvCodec, SdpSendCodec, SdpSendCodecParams};
pub use sdp::{MediaSessionId, MediaSessionState, SessionState, SrtpInfo};
pub use update::{handle_reinvite, refresh_invite_session};

struct AppData {
    contact: Contact,
    inbound_media_ip: IpAddr,
}

/// This layer is the application layer of the SIP endpoint and handles incoming INVITE requests
struct AppLayer {
    settings: Arc<Settings>,
    http_client: Arc<HttpClient>,
    shutdown: broadcast::Sender<()>,
    app_data: parking_lot::RwLock<Option<AppData>>,
}

impl AppLayer {
    fn set_app_data(&self, app_data: AppData) {
        *self.app_data.write() = Some(app_data);
    }
}

#[async_trait::async_trait]
impl Layer for AppLayer {
    fn name(&self) -> &'static str {
        "obelisk"
    }

    fn init(&mut self, endpoint: &mut EndpointBuilder) {
        endpoint.add_allow(Method::OPTIONS);
    }

    async fn receive(&self, endpoint: &Endpoint, request: MayTake<'_, IncomingRequest>) {
        let (contact, inbound_media_ip) = if let Some(app_data) = &*self.app_data.read() {
            (app_data.contact.clone(), app_data.inbound_media_ip)
        } else {
            log::warn!("Got request before contact is ready");
            return;
        };

        match request.line.method.clone() {
            Method::OPTIONS => {
                options::handle_options_request(endpoint, request.take(), &contact).await;
            }
            Method::INVITE => {
                if let Err(e) = self
                    .handle_invite(endpoint, request.take(), contact, inbound_media_ip)
                    .await
                {
                    log::error!("failed to handle invite {e:?}");
                }
            }
            _ => {}
        }
    }
}

impl AppLayer {
    /// Handle an incoming INVITE request and run until an error occurs or the callee hangs up
    async fn handle_invite(
        &self,
        endpoint: &Endpoint,
        mut request: IncomingRequest,
        contact: Contact,
        inbound_media_ip: IpAddr,
    ) -> Result<()> {
        let mut media_session = SessionState::empty(self.settings.clone(), inbound_media_ip);

        let mut changes = vec![];

        if let Ok(content_type) = request.headers.get_named::<ContentType>() {
            // Save body before handing the request to the acceptor
            let sdp_offer = request.body.clone();

            if content_type.0 != "application/sdp" {
                let tsx = endpoint.create_server_inv_tsx(&mut request);

                let response = endpoint.create_response(
                    &request,
                    StatusCode::NOT_ACCEPTABLE,
                    Some(BytesStr::from_static("Invalid Content Type")),
                );

                tsx.respond_failure(response).await?;
                bail!("Invalid invite content type {:?}", content_type.0);
            }

            changes = media_session
                .receive_offer(sdp_offer)
                .await
                .context("Failed to handle INVITE application/sdp body")?;
        }

        let dialog = Dialog::new_server(endpoint.clone(), &request, contact.clone())?;
        dialog.register_usage(options::OptionsUsage { contact });

        let fast_picture_update_notify = broadcast::Sender::new(1);

        let _usage = dialog.register_usage(media_control::MediaControlUsage {
            notify_picture_fast_update: fast_picture_update_notify.clone(),
        });

        let name = {
            let uri = match self.settings.sip.display_name_source {
                DisplayNameSource::Contact => &dialog.peer_contact.uri,
                DisplayNameSource::From => &dialog.peer_fromto.uri,
            };

            if let Some(name) = &uri.name {
                Some(name.clone())
            } else {
                match &uri.uri.user_part {
                    SipUriUserPart::Empty | SipUriUserPart::UserPw(_) => None,
                    SipUriUserPart::User(user) => Some(user.clone()),
                }
            }
        };

        let mut acceptor = InviteAcceptor::new(dialog, request);
        acceptor.timer_config().refresher = Refresher::Uas;

        // Send 100 Trying response to  avoid retransmits while processing the invite's SDP.
        let response = acceptor.create_response(StatusCode::TRYING, None).await?;
        acceptor.respond_provisional(response).await?;

        // Respond with 200 OK
        let mut response = acceptor.create_response(StatusCode::OK, None).await?;
        response
            .msg
            .headers
            .insert(Name::CONTENT_TYPE, "application/sdp");

        if changes.is_empty() {
            media_session
                .setup_offer()
                .await
                .context("Failed to setup media offer")?;
        }

        response.msg.body = media_session.build_sdp().to_string().into();

        let (session, ack) = acceptor.respond_success(response).await?;

        if changes.is_empty() {
            changes = media_session
                .receive_answer(ack.body)
                .context("Failed to handle SDP answer")?;
        }

        // Create a media pipeline
        let (mut media, track_controller) =
            MediaPipeline::new(self.settings.clone(), fast_picture_update_notify);

        update::apply_changes(&mut media, &mut media_session, changes).await?;

        // SIP Call established -> loop and handle events

        // Sleep about 1 second to wait for the media session
        // to work and then play the welcome track.
        sleep(Duration::from_secs(1)).await;

        track_controller.play_track(self.settings.language, Track::WelcomeConferenceId);

        let mut signaling = Signaling::new(
            self.settings.clone(),
            self.http_client.clone(),
            name,
            session,
            media,
            media_session,
            track_controller,
            self.shutdown.subscribe(),
        );

        Box::pin(signaling.run()).await?;

        Ok(())
    }
}

/// Application main-task
///
/// Run until the SIP registration failed or a SIGTERM was received
///
/// On shutdown it sends a shutdown signal to all active tasks shutting down all calls gracefully
/// and un-registering from the SIP registrar
#[allow(clippy::too_many_lines)]
pub(crate) async fn run(settings: Arc<Settings>) -> Result<()> {
    if let Some(MonitoringSettings { port, addr }) = settings.monitoring {
        start_probe(addr, port, ServiceState::Up).await?;
    }

    let mut builder = Endpoint::builder();

    builder.user_agent(format!("OpenTalk Obelisk {}", env!("CARGO_PKG_VERSION")));

    builder.add_layer(DialogLayer::default());
    builder.add_layer(InviteLayer::default());

    let (shutdown, _) = broadcast::channel(1);

    let http_client = HttpClient::discover(&settings.auth)
        .await
        .context("failed to discover oidc issuer")?;

    builder.add_layer(AppLayer {
        settings: settings.clone(),
        http_client: Arc::new(http_client),
        shutdown: shutdown.clone(),
        app_data: parking_lot::RwLock::new(None),
    });

    // Add TCP connector to allow the endpoint to create TCP connections
    builder.add_transport_factory(Arc::new(TcpConnector::default()));

    // Add TLS connector to allow the endpoint to create TLS connections
    builder.add_transport_factory(Arc::new(tokio_native_tls::TlsConnector::from(
        native_tls::TlsConnector::new()?,
    )));

    // Bind TCP transport if requested
    if settings.sip.transport.listen_tcp() {
        TcpListener::new()
            .spawn(
                &mut builder,
                format!("{}:{}", settings.sip.addr, settings.sip.port),
            )
            .await
            .context("Failed to bind TCP socket")?;
    }

    // Bind UDP transport if requested
    let udp_transport = if settings.sip.transport.listen_udp() {
        // Create a UDP socket to receive messages
        let transport = Udp::spawn(
            &mut builder,
            format!("{}:{}", settings.sip.addr, settings.sip.port),
        )
        .await
        .context("Failed to bind UDP Socket")?;

        Some(transport)
    } else {
        None
    };

    let endpoint = builder.build();

    // Find obelisk's public address using the UDP transport, when a stun-server is configured
    let pub_addr = if let Some((stun_server, udp_transport)) =
        settings.sip.stun_server.as_ref().zip(udp_transport)
    {
        let stun_server = if stun_server.contains(':') {
            Cow::Borrowed(stun_server)
        } else {
            Cow::Owned(format!("{stun_server}:3478"))
        };

        let stun_server: SocketAddr = lookup_host(stun_server.as_str())
            .await?
            .find(|addr| addr.is_ipv4() == udp_transport.bound().is_ipv4())
            .with_context(|| format!("configured stun-server '{stun_server}' yielded no addresses that are compatible with the configured bound address"))?;

        let pub_addr = endpoint
            .discover_public_address(stun_server, &udp_transport)
            .await?;

        log::info!("Discovered public address '{}'", pub_addr.ip());

        pub_addr
    } else if let Some(inbound_addr) = settings.sip.inbound_addr {
        inbound_addr
    } else {
        let pub_addr = lookup_host((settings.sip.addr.as_str(), settings.sip.port))
            .await?
            .next()
            .context("failed to resolve local addr")?;

        log::info!("Using {} as public address", pub_addr.ip());

        pub_addr
    };

    let id = generate_id_uri(&settings, pub_addr).context("Failed to generate id uri")?;
    let contact =
        generate_contact_uri(&settings, pub_addr).context("Failed to generate contact uri")?;

    // set app data
    let inbound_media_ip = settings.sip.inbound_media_ip.unwrap_or(pub_addr.ip());
    endpoint.layer::<AppLayer>().set_app_data(AppData {
        inbound_media_ip,
        contact: Contact::new(NameAddr::uri(contact.clone())),
    });

    if let Some(registrar) = &settings.sip.registrar {
        register_with_registrar(
            endpoint.clone(),
            pub_addr,
            id,
            contact,
            settings.sip.username.clone(),
            registrar.clone(),
            shutdown.subscribe(),
        )
        .await?;
    } else {
        log::info!("Incomplete SIP registrar settings, not registering anywhere.");
    }

    let mut sigterm =
        signal(SignalKind::terminate()).context("Failed to create SIGTERM signal handler")?;
    let ctrl_c = tokio::signal::ctrl_c();

    tokio::select! {
        _ = sigterm.recv() => {}
        _ = ctrl_c => {}
    }

    // ==== SHUTDOWN SEQUENCE

    // Send shutdown signal
    shutdown.send(()).ok();

    // Wait up to 10 seconds for all tasks to exit
    for _ in 0..10 {
        let receiver_count = shutdown.receiver_count();

        if receiver_count > 0 {
            log::info!("Waiting for {receiver_count} tasks to exit");

            sleep(Duration::from_secs(1)).await;
        } else {
            break;
        }
    }

    Ok(())
}

fn generate_id_uri(settings: &Arc<Settings>, pub_addr: SocketAddr) -> Result<SipUri> {
    // Overridden by config
    if let Some(id) = &settings.sip.id {
        return id
            .parse()
            .context("Configured SIP id is an invalid SIP uri");
    }

    let host = if let Some(registrar_settings) = &settings.sip.registrar {
        registrar_settings
            .registrar
            .trim_start_matches("sip:")
            .trim_start_matches("sips:")
            .to_owned()
    } else {
        pub_addr.to_string()
    };

    let uri = if let Some(username) = &settings.sip.username {
        format!("sip:{username}@{host}")
    } else {
        format!("sip:{host}")
    };

    uri.parse()
        .with_context(|| format!("Failed to parse generated id uri {uri:?}"))
}

fn generate_contact_uri(settings: &Arc<Settings>, pub_addr: SocketAddr) -> Result<SipUri> {
    // Overridden by config
    if let Some(contact) = &settings.sip.contact {
        return contact
            .parse()
            .context("Configured SIP Contact is an invalid SIP uri");
    }

    let uri = if let Some(username) = &settings.sip.username {
        format!("sip:{username}@{pub_addr}")
    } else {
        format!("sip:{pub_addr}")
    };

    uri.parse()
        .with_context(|| format!("Failed to parse generated contact uri {uri:?}"))
}

/// Try to register with a SIP registrar, then spawn a task to keep that registration up
async fn register_with_registrar(
    endpoint: Endpoint,
    pub_addr: SocketAddr,
    id: SipUri,
    contact: SipUri,
    username: Option<String>,
    settings: SipRegistrarSettings,
    mut shutdown: broadcast::Receiver<()>,
) -> Result<()> {
    const REGISTRATION_EXPIRY: Duration = Duration::from_secs(300);

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

    let mut registration = Registration::new(
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

    // Create binding
    let mut target = TargetTransportInfo {
        via_host_port: Some(pub_addr.into()),
        transport: Some((transport.clone(), target_addr)),
    };

    register(
        &endpoint,
        &mut target,
        &mut registration,
        &mut authenticator,
        false,
    )
    .await?;

    // Ping the sip server in an intervall to not loose the NAT binding
    tokio::spawn(nat_keep_alive(
        transport.clone(),
        target_addr,
        settings.nat_ping_delta,
    ));

    // Spawn a task that keeps the registration active
    tokio::spawn(async move {
        loop {
            let shutdown_recv = pin!(shutdown.recv());

            // Wait for either the the register interval to expire or the application shutdown signal
            tokio::select! {
                () = registration.wait_for_expiry() => {
                    // fallthrough to refresh binding
                }
                _ = shutdown_recv => {
                    // break out of loop to remove binding and exit the task
                    break;
                }
            }

            // Refresh binding
            if let Err(e) = register(
                &endpoint,
                &mut target,
                &mut registration,
                &mut authenticator,
                false,
            )
            .await
            {
                set_service_state(ServiceState::Up);
                log::error!("Failed to refresh binding, {e:?}");
            }
        }

        // Remove binding
        if let Err(e) = register(
            &endpoint,
            &mut target,
            &mut registration,
            &mut authenticator,
            true,
        )
        .await
        {
            set_service_state(ServiceState::Up);
            log::error!("Failed to remove binding, {e:?}");
        }
    });

    Ok(())
}

/// Send a register request and handle authentication using the given session and credentials
async fn register(
    endpoint: &Endpoint,
    target: &mut TargetTransportInfo,
    registration: &mut Registration,
    authenticator: &mut DigestAuthenticator,
    remove_binding: bool,
) -> Result<()> {
    loop {
        let mut request = registration.create_register(remove_binding);
        request.headers.insert_named(endpoint.allowed());
        request
            .headers
            .edit(Name::TO, |to: &mut FromTo| to.tag = None)?;

        authenticator.authorize_request(&mut request.headers);

        let mut transaction = endpoint.send_request(request, target).await?;

        let response = transaction.receive_final().await?;

        let response_code = response.line.code;

        match response_code.into_u16() {
            200..=299 => {
                if !remove_binding {
                    registration.receive_success_response(response);
                }
                set_service_state(ServiceState::Ready);
                return Ok(());
            }
            401 | 407 => authenticator.handle_rejection(
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
                if !registration.receive_error_response(response) {
                    bail!("Registration failed with code '{response_code:?}'");
                }
            }
            _ => bail!("Registration failed with code '{response_code:?}'"),
        }
    }
}

async fn nat_keep_alive(tp: TpHandle, target: SocketAddr, interval_delta: Duration) {
    let mut interval = interval(interval_delta);

    loop {
        if let Err(e) = tp.send(b"\r\n", target).await {
            log::error!("failed to send keep-alive, {e}");
        }

        interval.tick().await;
    }
}

fn maybe_add_sip_scheme(i: &str) -> Cow<'_, str> {
    if i.starts_with("sip:") || i.starts_with("sips:") {
        Cow::Borrowed(i)
    } else {
        Cow::Owned(format!("sip:{i}"))
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::http::HttpClient;
use crate::media::{MediaPipeline, Track};
use crate::settings::{Settings, SipRegistrarSettings};
use crate::signaling::Signaling;
use anyhow::{bail, Context, Result};
use bytesstr::BytesStr;
use sip_auth::digest::{DigestAuthenticator, DigestCredentials};
use sip_auth::{CredentialStore, RequestParts, UacAuthSession};
use sip_core::transport::tcp::TcpConnector;
use sip_core::transport::udp::Udp;
use sip_core::transport::{TargetTransportInfo, TpHandle};
use sip_core::{Endpoint, IncomingRequest, Layer, LayerKey, MayTake};
use sip_types::header::typed::{Contact, ContentType};
use sip_types::print::AppendCtx;
use sip_types::uri::sip::{SipUri, UserPart};
use sip_types::uri::{NameAddr, Uri};
use sip_types::{Code, Method, Name};
use sip_ua::dialog::{Dialog, DialogLayer};
use sip_ua::invite::acceptor::Acceptor;
use sip_ua::invite::InviteLayer;
use sip_ua::register::Registration;
use std::borrow::Cow;
use std::net::{IpAddr, SocketAddr};
use std::pin::pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::lookup_host;
use tokio::sync::{broadcast, Notify};
use tokio::time::{interval, sleep};

struct AppData {
    contact: Contact,
    pub_ip: IpAddr,
}

/// This layer is the application layer of the SIP endpoint and handles incoming INVITE requests
struct AppLayer {
    settings: Arc<Settings>,
    http_client: Arc<HttpClient>,
    shutdown: broadcast::Sender<()>,
    app_data: parking_lot::RwLock<Option<AppData>>,
    dialog_layer: LayerKey<DialogLayer>,
    invite_layer: LayerKey<InviteLayer>,
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

    async fn receive(&self, endpoint: &Endpoint, request: MayTake<'_, IncomingRequest>) {
        let request = if request.line.method == Method::INVITE {
            request.take()
        } else {
            return;
        };

        let (contact, pub_ip) = if let Some(app_data) = &*self.app_data.read() {
            (app_data.contact.clone(), app_data.pub_ip)
        } else {
            log::warn!("Got request before contact is ready");
            return;
        };

        if let Err(e) = self.handle_invite(endpoint, request, contact, pub_ip).await {
            log::error!("failed to handle invite {:?}", e);
        }
    }
}

impl AppLayer {
    /// Handle an incoming INVITE request and run until an error occurs or the callee hangs up
    async fn handle_invite(
        &self,
        endpoint: &Endpoint,
        request: IncomingRequest,
        contact: Contact,
        pub_ip: IpAddr,
    ) -> Result<()> {
        // Save body and display name before handing the request to the acceptor
        let sdp_offer = request.body.clone();

        let content_type: ContentType = request.headers.get_named()?;

        if content_type.0 != "application/sdp" {
            let tsx = endpoint.create_server_inv_tsx(&request);

            let response = endpoint.create_response(
                &request,
                Code::NOT_ACCEPTABLE,
                Some(BytesStr::from_static("Invalid Content Type")),
            );

            tsx.respond_failure(response).await?;
            bail!("Invalid invite content type {:?}", content_type.0);
        }

        let dialog = Dialog::new_server(endpoint.clone(), self.dialog_layer, &request, contact)?;

        let name = if let Some(name) = &dialog.peer_contact.uri.name {
            Some(name.clone())
        } else {
            dialog
                .peer_contact
                .uri
                .uri
                .downcast_ref::<SipUri>()
                .and_then(|uri| match &uri.user_part {
                    UserPart::Empty => None,
                    UserPart::User(user) => Some(user.clone()),
                    UserPart::UserPw(_) => None,
                })
        };

        let mut acceptor = Acceptor::new(dialog, self.invite_layer, request)?;

        // Send 100 Trying response to avoid retransmits while processing the invite's SDP.
        let response = acceptor.create_response(Code::TRYING, None).await?;
        acceptor.respond_provisional(response).await?;

        // Hand the incoming SDP to the media task
        let ready_to_send = Arc::new(Notify::new());
        let (handle, track_controller, sdp_answer) =
            match MediaPipeline::new(pub_ip, sdp_offer, ready_to_send.clone()).await {
                Ok(answer) => answer,
                Err(e) => {
                    log::error!("Failed to create GStreamer pipeline, {e:?}");

                    return self.internal_server_error(acceptor).await;
                }
            };

        // Respond with 200 OK
        let mut response = acceptor.create_response(Code::OK, None).await?;

        response.msg.body = sdp_answer.into();
        response
            .msg
            .headers
            .insert(Name::CONTENT_TYPE, "application/sdp");

        let (session, _) = acceptor.respond_success(response).await?;

        // SIP Call established -> loop and handle events

        // Wait for the client to be ready to accept playback
        ready_to_send.notified().await;
        // Sleep about 1 second to wait for the media session
        // to work and then play the welcome track.
        sleep(Duration::from_secs(1)).await;

        track_controller.play_track(Track::WelcomeConferenceId);

        let mut signaling = Signaling::new(
            self.settings.clone(),
            self.http_client.clone(),
            name,
            session,
            handle,
            track_controller,
            self.shutdown.subscribe(),
        );

        signaling.run().await?;

        Ok(())
    }

    /// Consume the acceptor by terminating the transaction with an 500 Internal Server Error
    async fn internal_server_error(&self, acceptor: Acceptor) -> Result<()> {
        let response = acceptor
            .create_response(Code::SERVER_INTERNAL_ERROR, None)
            .await?;

        acceptor.respond_failure(response).await?;

        Ok(())
    }
}

/// Application main-task
///
/// Run until the SIP registration failed or a SIGTERM was received
///
/// On shutdown it sends a shutdown signal to all active tasks shutting down all calls gracefully
/// and un-registering from the SIP registrar
pub(crate) async fn run(settings: Arc<Settings>) -> Result<()> {
    let mut builder = Endpoint::builder();

    let dialog_layer = builder.add_layer(DialogLayer::default());
    let invite_layer = builder.add_layer(InviteLayer::default());

    let (shutdown, _) = broadcast::channel(1);

    let http_client = HttpClient::discover(&settings.auth)
        .await
        .context("failed to discover oidc issuer")?;

    let app_layer = builder.add_layer(AppLayer {
        settings: settings.clone(),
        http_client: Arc::new(http_client),
        shutdown: shutdown.clone(),
        app_data: parking_lot::RwLock::new(None),
        dialog_layer,
        invite_layer,
    });

    // Create a UDP socket to receive messages
    let transport = Udp::spawn(
        &mut builder,
        format!("{}:{}", settings.sip.addr, settings.sip.port),
    )
    .await?;

    // Add TCP connector to allow the endpoint to create TCP connections
    builder.add_transport_factory(Arc::new(TcpConnector::default()));

    // Add TLS connector to allow the endpoint to create TLS connections
    builder.add_transport_factory(Arc::new(tokio_native_tls::TlsConnector::from(
        native_tls::TlsConnector::new()?,
    )));

    let endpoint = builder.build();

    // Find obelisk's public address using the UDP transport, when a stun-server is configured
    let pub_addr = if let Some(stun_server) = settings.sip.stun_server.as_ref() {
        let stun_server: SocketAddr = lookup_host(stun_server.as_str())
            .await?
            .find(|addr| addr.is_ipv4() == transport.bound().is_ipv4())
            .with_context(|| format!("configured stun-server '{stun_server}' yielded no addresses that are compatible with the configured bound address"))?;

        let pub_addr = endpoint
            .discover_public_address(stun_server, &transport)
            .await?;

        log::info!("Discovered public address '{}'", pub_addr.ip());

        pub_addr
    } else {
        let pub_addr = lookup_host((settings.sip.addr.as_str(), settings.sip.port))
            .await?
            .next()
            .context("failed to resolve local addr")?;

        log::info!("Using {} as public address", pub_addr.ip());

        pub_addr
    };

    // Construct this SIP endpoints SIP uri
    let id = if let Some(id) = &settings.sip.id {
        endpoint
            .parse_uri(id)
            .context("Failed to parse `id` sip uri")?
    } else if let Some(username) = &settings.sip.username {
        endpoint.parse_uri(format!("sip:{}@{}", username, pub_addr))?
    } else {
        endpoint.parse_uri(format!("sip:{}", pub_addr))?
    };

    // set app data
    endpoint[app_layer].set_app_data(AppData {
        pub_ip: pub_addr.ip(),
        contact: Contact::new(NameAddr::uri(id.clone())),
    });

    if let Some(registrar) = &settings.sip.registrar {
        register_with_registrar(
            endpoint.clone(),
            pub_addr,
            id,
            settings.sip.username.clone(),
            registrar.clone(),
            shutdown.subscribe(),
        )
        .await?;
    } else {
        log::info!("Incomplete SIP registrar settings, not registering anywhere.");
    }

    tokio::signal::ctrl_c().await?;

    // ==== SHUTDOWN SEQUENCE

    // Send shutdown signal
    shutdown.send(()).ok();

    // Wait up to 10 seconds for all tasks to exit
    for _ in 0..10 {
        let receiver_count = shutdown.receiver_count();

        if receiver_count > 0 {
            log::info!("Waiting for {} tasks to exit", receiver_count);

            sleep(Duration::from_secs(1)).await;
        } else {
            break;
        }
    }

    Ok(())
}

/// Try to register with a SIP registrar, then spawn a task to keep that registration up
async fn register_with_registrar(
    endpoint: Endpoint,
    pub_addr: SocketAddr,
    id: Box<dyn Uri>,
    username: Option<String>,
    settings: SipRegistrarSettings,
    mut shutdown: broadcast::Receiver<()>,
) -> Result<()> {
    let registrar_uri = maybe_add_sip_scheme(&settings.registrar);
    let registrar_uri = endpoint
        .parse_uri(&registrar_uri)
        .context("Failed to parse registrar URI")?;

    // Check if we send the request to an outbound proxy or directly to the registrar uri
    let target_uri = if let Some(outbound_proxy) = settings.outbound_proxy {
        let outbound_proxy = maybe_add_sip_scheme(&outbound_proxy);

        endpoint.parse_uri(&outbound_proxy)?
    } else {
        registrar_uri.clone()
    };

    // Select a transport for the URI to which to send the REGISTER to
    let (transport, target_addr) =
        endpoint
            .select_transport(&*target_uri)
            .await
            .with_context(|| {
                format!(
                    "Failed to select transport for '{}'",
                    target_uri.default_print_ctx()
                )
            })?;

    let mut registration = Registration::new(NameAddr::uri(id), registrar_uri);

    let mut authenticator = DigestAuthenticator::default();
    authenticator.enforce_qop = settings.enforce_qop;

    let mut auth_sess = UacAuthSession::new(authenticator);
    let mut credentials = CredentialStore::new();

    // Add username & password credentials if both are set
    if let Some((username, password)) = username.zip(settings.password) {
        let credential = DigestCredentials::new(username, password);

        if let Some(realm) = settings.realm {
            credentials.add_for_realm(realm, credential);
        } else {
            credentials.set_default(credential);
        }
    }

    // Create binding
    let mut target = TargetTransportInfo {
        via_host_port: Some(pub_addr.into()),
        transport: Some((transport.clone(), target_addr)),
    };

    register(
        &endpoint,
        &mut target,
        &mut registration,
        &mut auth_sess,
        &credentials,
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
                _ = registration.wait_for_expiry() => {
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
                &mut auth_sess,
                &credentials,
                false,
            )
            .await
            {
                log::error!("Failed to refresh binding, {e:?}");
            }
        }

        // Remove binding
        if let Err(e) = register(
            &endpoint,
            &mut target,
            &mut registration,
            &mut auth_sess,
            &credentials,
            true,
        )
        .await
        {
            log::error!("Failed to remove binding, {e:?}")
        }
    });

    Ok(())
}

/// Send a register request and handle authentication using the given session and credentials
async fn register(
    endpoint: &Endpoint,
    target: &mut TargetTransportInfo,
    registration: &mut Registration,
    auth_sess: &mut UacAuthSession,
    credentials: &CredentialStore,
    remove_binding: bool,
) -> Result<()> {
    loop {
        let mut request = registration.create_register(remove_binding);
        request.headers.insert_named(endpoint.allowed());

        auth_sess.authorize_request(&mut request.headers);

        let mut transaction = endpoint.send_request(request, target).await?;

        let response = transaction.receive_final().await?;

        let response_code = response.line.code;

        match response_code.into_u16() {
            200..=299 => {
                if !remove_binding {
                    registration.receive_success_response(response);
                }

                return Ok(());
            }
            401 | 407 => auth_sess.handle_authenticate(
                &response.headers,
                credentials,
                RequestParts {
                    line: &transaction.request().msg.line,
                    headers: &transaction.request().msg.headers,
                    body: &transaction.request().msg.body,
                },
            )?,
            400..=499 if !remove_binding => {
                if !registration.receive_error_response(response) {
                    bail!("Registration failed with code '{:?}'", response_code);
                }
            }
            _ => bail!("Registration failed with code '{:?}'", response_code),
        }
    }
}

async fn nat_keep_alive(tp: TpHandle, target: SocketAddr, interval_delta: Duration) {
    let mut interval = interval(interval_delta);

    loop {
        if let Err(e) = tp.send(b"\r\n", target).await {
            log::error!("failed to send keep-alive, {}", e);
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

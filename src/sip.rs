// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::http::HttpClient;
use crate::media::{self, MediaTaskHandle, Track};
use crate::settings::Settings;
use crate::signaling::Signaling;
use anyhow::{bail, Context, Result};
use bytesstr::BytesStr;
use gst::glib;
use sip_auth::digest::{DigestAuthenticator, DigestCredentials};
use sip_auth::{CredentialStore, RequestParts, UacAuthSession};
use sip_core::transport::udp::Udp;
use sip_core::transport::{TargetTransportInfo, TpHandle};
use sip_core::{Endpoint, IncomingRequest, Layer, LayerKey, MayTake};
use sip_types::header::typed::{Contact, ContentType};
use sip_types::uri::sip::{SipUri, UserPart};
use sip_types::uri::NameAddr;
use sip_types::{Code, Method, Name};
use sip_ua::dialog::{Dialog, DialogLayer};
use sip_ua::invite::acceptor::Acceptor;
use sip_ua::invite::InviteLayer;
use sip_ua::register::Registration;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::lookup_host;
use tokio::sync::broadcast;
use tokio::time::{interval, sleep};

struct AppData {
    contact: Contact,
    pub_ip: IpAddr,
    main_ctx: glib::MainContext,
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

        let (contact, handle) = if let Some(app_data) = &*self.app_data.read() {
            let contact = app_data.contact.clone();
            let handle = media::spawn_task(app_data.pub_ip, &app_data.main_ctx);

            (contact, handle)
        } else {
            log::warn!("Got request before contact is ready");
            return;
        };

        if let Err(e) = self.handle_invite(endpoint, request, contact, handle).await {
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
        handle: MediaTaskHandle,
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
        let (track_controller, sdp_answer) = match handle.send_sip_sdp_offer(sdp_offer).await {
            Ok(answer) => answer,
            Err(_) => return self.internal_server_error(acceptor).await,
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
pub(crate) async fn run(main_loop: glib::MainLoop, settings: Arc<Settings>) -> Result<()> {
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
        shutdown,
        app_data: parking_lot::RwLock::new(None),
        dialog_layer,
        invite_layer,
    });

    let transport = Udp::spawn(
        &mut builder,
        format!("{}:{}", settings.sip.addr, settings.sip.port),
    )
    .await?;

    let endpoint = builder.build();

    let pub_addr = if let Some(stun_server) = settings.sip.stun_server.as_ref() {
        let stun_server: Vec<SocketAddr> = lookup_host(stun_server.as_str()).await?.collect();
        let pub_addr = endpoint
            .discover_public_address(&stun_server, &transport)
            .await?;

        log::info!("Discovered public address {}", pub_addr.ip());

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
    let id = endpoint.parse_uri(format!("sip:{}@{}", settings.sip.username, pub_addr))?;

    // set app data
    endpoint[app_layer].set_app_data(AppData {
        pub_ip: pub_addr.ip(),
        main_ctx: main_loop.context(),
        contact: Contact::new(NameAddr::uri(id.clone())),
    });

    let mut target = TargetTransportInfo {
        via_host_port: Some(pub_addr.into()),
        transport: Some(transport.clone()),
        destination: vec![],
    };

    let registrar = endpoint.parse_uri(&settings.sip.registrar)?;

    let mut registration = Registration::new(NameAddr::uri(id), registrar);

    let mut authenticator = DigestAuthenticator::default();
    authenticator.enforce_qop = settings.sip.enforce_qop;

    let mut auth_sess = UacAuthSession::new(authenticator);
    let mut credentials = CredentialStore::new();

    credentials.add_for_realm(
        settings.sip.realm.clone(),
        DigestCredentials::new(settings.sip.username.clone(), settings.sip.password.clone()),
    );

    let mut ctrl_c = Box::pin(tokio::signal::ctrl_c());

    let mut nat_keep_alive_spawned = false;

    loop {
        let mut request = registration.create_register(false);
        request.headers.insert_named(endpoint.allowed());

        auth_sess.authorize_request(&mut request.headers);

        let mut transaction = endpoint.send_request(request, &mut target).await?;

        let response = transaction.receive_final().await?;

        match response.line.code.into_u16() {
            200..=299 => {
                registration.receive_success_response(response);

                // On successful registration spawn task that will send CRLF keep alive packets
                if !nat_keep_alive_spawned {
                    nat_keep_alive_spawned = true;
                    tokio::spawn(nat_keep_alive(
                        transport.clone(),
                        target.destination.clone(),
                        settings.sip.nat_ping_delta,
                    ));
                }

                tokio::select! {
                    _ = registration.wait_for_expiry() => {
                        continue;
                    }
                    _ = &mut ctrl_c => {
                        break;
                    }
                };
            }
            401 | 407 => auth_sess.handle_authenticate(
                &response.headers,
                &credentials,
                RequestParts {
                    line: &transaction.request().msg.line,
                    headers: &transaction.request().msg.headers,
                    body: &transaction.request().msg.body,
                },
            )?,
            _ => {
                bail!("Registration failed!");
            }
        }
    }

    // ==== SHUTDOWN SEQUENCE

    let shutdown = &endpoint[app_layer].shutdown;

    // Send shutdown signal
    shutdown.send(()).ok();

    drop(ctrl_c);

    // Then remove binding from registrar
    loop {
        let mut request = registration.create_register(true);
        auth_sess.authorize_request(&mut request.headers);

        let mut transaction = endpoint.send_request(request, &mut target).await?;

        let response = transaction.receive_final().await?;

        if let 401 | 407 = response.line.code.into_u16() {
            auth_sess.handle_authenticate(
                &response.headers,
                &credentials,
                RequestParts {
                    line: &transaction.request().msg.line,
                    headers: &transaction.request().msg.headers,
                    body: &transaction.request().msg.body,
                },
            )?;
        } else {
            break;
        }
    }

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

    log::info!("Obelisk exiting, bye!");

    main_loop.quit();

    Ok(())
}

async fn nat_keep_alive(tp: TpHandle, target: Vec<SocketAddr>, interval_delta: Duration) {
    let mut interval = interval(interval_delta);

    loop {
        if let Err(e) = tp.send(b"\r\n", &target).await {
            log::error!("failed to send keep-alive, {}", e);
        }

        interval.tick().await;
    }
}

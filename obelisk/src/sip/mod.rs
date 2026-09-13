// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    borrow::Cow,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use anyhow::{Context, Result, bail};
use bytesstr::BytesStr;
use ezk_sip_core::{Endpoint, EndpointBuilder, IncomingRequest, Layer, MayTake};
use ezk_sip_types::{
    Method, Name, StatusCode,
    header::typed::{Contact, ContentType, Refresher},
    uri::{NameAddr, SipUri, SipUriUserPart},
};
use ezk_sip_ua::{
    dialog::{Dialog, DialogLayer},
    invite::{InviteLayer, acceptor::InviteAcceptor},
};
use opentalk_client_signaling::{AuthConfig, OpenTalkClient, config::ClientConfig};
use service_probe::{ServiceState, start_probe};
use tokio::{
    net::lookup_host,
    signal::unix::{SignalKind, signal},
    sync::broadcast,
    time::sleep,
};

use crate::{
    media::{MediaPipeline, Track},
    settings::{DisplayNameSource, MonitoringSettings, Settings},
    signaling::Signaling,
    sip::registration::RegistrationTask,
};

mod codec;
mod media_control;
mod options;
mod registration;
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
    opentalk_client: Arc<OpenTalkClient>,
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
        let _options_usage = dialog.register_usage(options::OptionsUsage { contact });

        let fast_picture_update_notify = broadcast::Sender::new(1);

        let _media_control_usage = dialog.register_usage(media_control::MediaControlUsage {
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
            self.opentalk_client.clone(),
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

    let opentalk_client = OpenTalkClient::create(ClientConfig {
        auth: AuthConfig::ApiKey(settings.controller.api_key.clone()),
        controller: settings.controller.url.clone(),
    })
    .await
    .context("Failed to create OpenTalkClient")?;

    builder.add_layer(AppLayer {
        settings: settings.clone(),
        opentalk_client: Arc::new(opentalk_client),
        shutdown: shutdown.clone(),
        app_data: parking_lot::RwLock::new(None),
    });

    // Add TLS connector to allow the endpoint to create TLS connections
    builder.use_native_tls_connector(tokio_native_tls::TlsConnector::from(
        native_tls::TlsConnector::new()?,
    ));

    // Bind TCP transport if requested
    if settings.sip.transport.listen_tcp() {
        builder
            .listen_tcp(
                format!("{}:{}", settings.sip.addr, settings.sip.port)
                    .parse()
                    .context("Failed to parse socket-addr for TCP socket")?,
            )
            .await
            .context("Failed to bind TCP listener")?;
    }

    // Bind UDP transport if requested
    let udp_transport = if settings.sip.transport.listen_udp() {
        let transport = builder
            .bind_udp(
                format!("{}:{}", settings.sip.addr, settings.sip.port)
                    .parse()
                    .context("Failed to parse socket-addr for UDP socket")?,
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
        RegistrationTask::register_and_spawn(
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

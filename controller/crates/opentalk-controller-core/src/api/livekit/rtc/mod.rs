// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{str::FromStr as _, sync::Arc};

use actix_http::header::HeaderMap;
use actix_web::{
    HttpRequest, HttpResponse, get,
    web::{Data, Header, Payload, Query},
};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use livekit_api::access_token::Claims;
use opentalk_controller_service::controller_backend::roomserver::SignalingProxyBackend;
use opentalk_roomserver_types::livekit_proxy::{
    LiveKitProxyRequest, actix::LiveKitSocketAdapter, websocket::LiveKitSocketMessage,
};
use opentalk_roomserver_web_api::livekit_proxy::{self, LiveKitQuery};
use opentalk_types_api_v1::error::ApiError;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::StreamExt as _;

pub mod v1;
pub mod validate;

const CHANNEL_BUFFER_SIZE: usize = 32;

/// Opens a new LiveKit WebSocket connection for RTC communication.
///
/// This endpoint establishes a WebSocket connection to the upstream LiveKit server.
/// The access token can be provided either as a query parameter or in the `Authorization`
/// header with the `Bearer` scheme.
///
/// # Available paths
/// - `/livekit/rtc`
/// - `/livekit/rtc/v1`
#[utoipa::path(
    get,
    path = "/livekit/rtc",
    operation_id = "livekit_rtc",
    responses(
        (status = StatusCode::SWITCHING_PROTOCOLS, description = "Successfully upgraded connection to WebSocket for RTC communication"),
        (status = StatusCode::BAD_REQUEST, description = "Invalid token format or missing required token claims"),
        (status = StatusCode::UNAUTHORIZED, description = "Missing access token or invalid token scheme"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "An internal server error occurred"),
    ),
    params(
        ("access_token" = Option<String>, Query, description = "LiveKit access token (JWT) as query parameter. If not provided, must be in Authorization header with Bearer scheme")
    ),
    security(
        ("Livekit-Token" = [])
    ),
)]
#[tracing::instrument(level = "info", name = "/livekit/proxy/rtc", skip_all)]
#[get("rtc")]
pub async fn get(
    proxy: Data<Option<Arc<dyn SignalingProxyBackend>>>,
    req: HttpRequest,
    auth_header: Option<Header<Authorization<Bearer>>>,
    payload: Payload,
    query: Query<LiveKitQuery>,
    shutdown: Data<broadcast::Sender<()>>,
) -> Result<HttpResponse, ApiError> {
    proxy_socket(proxy, req, auth_header, payload, query, shutdown).await
}

pub(crate) async fn proxy_socket(
    proxy: Data<Option<Arc<dyn SignalingProxyBackend>>>,
    req: HttpRequest,
    auth_header: Option<Header<Authorization<Bearer>>>,
    payload: Payload,
    query: Query<LiveKitQuery>,
    shutdown: Data<broadcast::Sender<()>>,
) -> Result<HttpResponse, ApiError> {
    let proxy = proxy
        .as_ref() // Option as ref
        .as_ref() // Arc as ref
        .ok_or_else(|| ApiError::not_found().with_message("LiveKit proxy not available"))?;

    let access_token = extract_access_token(query, auth_header)?;

    // We do not verify the token since this is done by livekit. We only proxy the connection.
    let content = jsonwebtoken::dangerous::insecure_decode::<Claims>(access_token.as_bytes())
        .map_err(|err| {
            tracing::debug!("Failed to decode livekit token: {err}");
            ApiError::bad_request()
        })?;

    let (room_id, proxy_target) = livekit_proxy::parse_livekit_room_id(&content.claims.video.room)?;
    let (participant_id, connection_id) =
        livekit_proxy::parse_livekit_participant(&content.claims.sub)?;

    let raw_query = raw_query(&req);
    let headers = to_http1_headers(req.headers());
    let ws_request = LiveKitProxyRequest {
        raw_query,
        headers,
        room_id,
        proxy_target,
        participant_id,
        connection_id,
    };

    let upstream_socket = proxy.connect_upstream_socket(ws_request.clone()).await?;

    // Perform the WebSocket upgrade
    let (response, session, msg_stream) = actix_ws::handle(&req, payload).map_err(|err| {
        tracing::error!("WebSocket handshake failed: {err}");
        ApiError::internal().with_message("WebSocket handshake failed")
    })?;

    // Create channels for bridging between actix-ws and the WebSocketAdapter
    let (incoming_tx, incoming_rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
    let (outgoing_tx, mut outgoing_rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);

    let socket = LiveKitSocketAdapter::new(incoming_rx, outgoing_tx);

    let mut shutdown_rx = shutdown.subscribe();

    // Spawn the proxy task before connecting the downstream socket to ensure errors get propagated correctly should
    // the connection fail
    actix_web::rt::spawn(async move {
        let mut session = session;
        let mut msg_stream = msg_stream;

        loop {
            tokio::select! {
               // Incoming: client -> proxy -> livekit
               ws_msg = msg_stream.next() => {
                  match ws_msg {
                     Some(Ok(msg)) => {
                        if incoming_tx.send(Ok(msg)).await.is_err() {
                           // Receiver dropped, room task gone
                           break;
                        }
                     }
                     Some(Err(err)) => {
                        // Protocol error - forward it and stop
                        _ = incoming_tx.send(Err(err.into())).await;
                        break;
                     }
                     None => {
                        // WebSocket stream ended
                        break;
                     }
                  }
               }
               // Shutdown: close session when the server is shutting down
               _ = shutdown_rx.recv() => {
                  tracing::debug!("Shutdown signal received, closing livekit proxy WebSocket");
                  _ = session.close(None).await;
                  break;
               }
               // Outgoing: livekit -> proxy -> client
               out_msg = outgoing_rx.recv() => {
                  match out_msg {
                    Some(msg) => {
                       let result = match msg {
                          LiveKitSocketMessage::Text(text) => session.text(text).await,
                          LiveKitSocketMessage::Binary(bytes) => session.binary(bytes).await,
                          LiveKitSocketMessage::Ping(bytes) => session.ping(&bytes).await,
                          LiveKitSocketMessage::Pong(bytes) => session.pong(&bytes).await,
                          LiveKitSocketMessage::Close(close_frame) => {
                             _ = session.close(close_frame.map(Into::into)).await;
                             break;
                          },
                       };
                       if result.is_err() {
                          // Session closed unexpectedly
                          break;
                       }
                    },
                    None => {
                       // Sender dropped, close session
                       _ = session.close(None).await;
                       break;
                    },
                  }
               }
            }
        }
    });

    proxy
        .connect_downstream_socket(ws_request, upstream_socket, Box::new(socket))
        .await
        .inspect_err(|err| tracing::warn!("Failed to accept livekit socket: {err:?}"))?;

    Ok(response)
}

fn extract_access_token(
    query: Query<LiveKitQuery>,
    auth_header: Option<Header<Authorization<Bearer>>>,
) -> Result<String, ApiError> {
    if let Some(token) = query.into_inner().access_token {
        Ok(token)
    } else if let Some(header) = auth_header {
        Ok(header.into_inner().into_scheme().token().to_owned())
    } else {
        Err(ApiError::unauthorized())
    }
}

fn raw_query(req: &HttpRequest) -> Option<String> {
    if req.query_string().is_empty() {
        None
    } else {
        Some(req.query_string().to_owned())
    }
}

fn to_http1_headers(actix_headers: &HeaderMap) -> http::HeaderMap {
    actix_headers
        .iter()
        .filter_map(|(name, value)| {
            let name = http::HeaderName::from_str(name.as_str()).ok()?;
            let value = http::HeaderValue::from_bytes(value.as_bytes()).ok()?;

            Some((name, value))
        })
        .collect()
}

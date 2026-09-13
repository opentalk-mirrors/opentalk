// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling WebSocket endpoint for the internal roomserver

use std::sync::Arc;

use actix_web::{
    HttpRequest, HttpResponse, get,
    web::{Data, Path, Payload},
};
use futures::StreamExt as _;
use opentalk_controller_service::controller_backend::roomserver::SignalingProxyBackend;
use opentalk_roomserver_types::signaling::websocket::SignalingSocketMessage;
use opentalk_types_api_v1::error::ApiError;
use opentalk_types_common::roomserver::Token;
use tokio::sync::mpsc;

const CHANNEL_BUFFER_SIZE: usize = 32;

/// Establish a signaling WebSocket connection with the internal roomserver
///
/// This endpoint upgrades the HTTP connection to a WebSocket and attaches the
/// connection to the room task identified by the provided signaling token.
/// The token is obtained from a prior call to the `/v1/rooms/{room_id}/start`
/// or `/v1/rooms/{room_id}/start_invited` endpoint.
///
/// This endpoint is only available when the controller is configured with an
/// internal roomserver. Returns `404 Not Found` when using an external roomserver.
#[utoipa::path(
    operation_id = "signaling_ws",
    tag = "api::v1::signaling",
    params(
        ("token" = Token, description = "The signaling token obtained from a start endpoint"),
    ),
    responses(
        (
            status = StatusCode::SWITCHING_PROTOCOLS,
            description = "WebSocket connection successfully established",
        ),
        (
            status = StatusCode::NOT_FOUND,
            description = "The signaling token is invalid, expired, or the internal roomserver is not enabled",
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            description = "Failed to attach signaling socket to the room task",
        ),
    ),
    security(),
)]
#[get("/signaling/{token}")]
pub async fn get(
    req: HttpRequest,
    payload: Payload,
    token: Path<Token>,
    signaling: Data<Option<Arc<dyn SignalingProxyBackend>>>,
) -> Result<HttpResponse, ApiError> {
    let signaling = signaling
        .as_ref() // Option as ref
        .as_ref() // Arc as ref
        .ok_or_else(|| {
            ApiError::not_found()
                .with_message("Signaling endpoint is not available with an external roomserver")
        })?;

    let token = token.into_inner();

    // Consume the token before upgrading so we can still return an HTTP error
    let signaling_ctx = signaling.consume_signaling_token(&token).await?;

    // Perform the WebSocket upgrade
    let (response, session, msg_stream) = actix_ws::handle(&req, payload).map_err(|err| {
        tracing::error!("WebSocket handshake failed: {err}");
        ApiError::internal().with_message("WebSocket handshake failed")
    })?;

    // Create channels for bridging between actix-ws and the WebSocketAdapter
    let (incoming_tx, incoming_rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
    let (outgoing_tx, mut outgoing_rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);

    // Spawn a local task to bridge the actix-ws Session/MessageStream with the mpsc channels.
    // This task must run on the local thread (spawn_local) because Session and MessageStream are !Send.
    actix_web::rt::spawn(async move {
        let mut session = session;
        let mut msg_stream = msg_stream;

        loop {
            tokio::select! {
                // Incoming: read from WebSocket MessageStream, forward to incoming channel
                ws_msg = msg_stream.next() => {
                    match ws_msg {
                        Some(Ok(msg)) => {
                            if incoming_tx.send(Ok(msg)).await.is_err() {
                                // Receiver dropped, room task gone
                                break;
                            }
                        }
                        Some(Err(err)) => {
                            // Protocol error — forward it and stop
                            let _ = incoming_tx.send(Err(err)).await;
                            break;
                        }
                        None => {
                            // WebSocket stream ended
                            break;
                        }
                    }
                }
                // Outgoing: read from outgoing channel, send via Session
                out_msg = outgoing_rx.recv() => {
                    match out_msg {
                        Some(msg) => {
                            let result = match msg {
                                SignalingSocketMessage::Text(text) => session.text(text).await,
                                SignalingSocketMessage::Binary(bytes) => session.binary(bytes).await,
                                SignalingSocketMessage::Ping(bytes) => session.ping(&bytes).await,
                                SignalingSocketMessage::Pong(bytes) => session.pong(&bytes).await,
                                SignalingSocketMessage::Close(frame) => {
                                    let reason = frame.map(|f| actix_ws::CloseReason {
                                        code: actix_ws::CloseCode::from(
                                            f.code
                                        ),
                                        description: Some(f.reason),
                                    });
                                    // close() consumes the session, so break after calling it
                                    let _ = session.close(reason).await;
                                    break;
                                }
                            };
                            if result.is_err() {
                                // Session closed
                                break;
                            }
                        }
                        None => {
                            // Sender dropped, room task done — close session
                            let _ = session.close(None).await;
                            break;
                        }
                    }
                }
            }
        }
    });

    // Attach the signaling connection to the room task via the roomserver backend
    signaling
        .accept_signaling_connection(signaling_ctx, incoming_rx, outgoing_tx)
        .await?;

    Ok(response)
}

// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! This module provides two way communication between the room server and it's clients (e.g.
//! web-app).

use std::fmt::Debug;

use async_trait::async_trait;
use axum::{
    extract::{
        Path, State,
        ws::{WebSocket, WebSocketUpgrade},
    },
    http::Method,
    response::{IntoResponse, Response},
    routing::any,
};
use opentalk_roomserver_types::{
    client_parameters::ClientParameters, signaling::signaling_context::SignalingClientContext,
};
use opentalk_types_common::{rooms::RoomId, roomserver::Token};
use tracing::{Instrument as _, Span};

use super::Router;
use crate::v1::cors;

pub(crate) fn routes<B: SignalingBackend + 'static>(state: B) -> Router<B> {
    Router::new()
        .route("/signaling/{token}", any(open_signaling_socket::<B>))
        .layer(cors::cors_layer(
            state,
            [Method::GET, Method::OPTIONS, Method::HEAD],
        ))
}

/// Opens a new signaling websocket connection.
#[utoipa::path(
    get,
    path = "/signaling/{token}",
    responses(
        (status = StatusCode::SWITCHING_PROTOCOLS, description = "Successfully upgraded connection to WebSocket"),
        (status = StatusCode::NOT_FOUND, description = "The requested room does not exist"),
        (status = StatusCode::UNAUTHORIZED, description = "The token was invalid"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "An internal server error occurred"),
    ),
    params(
        ("token" = Token, Path, description = "The roomserver token that authorizes the user")
    ),
    security(),
    )]
#[tracing::instrument(name = "/signaling/{token}", level = "info", skip_all, fields(opentalk.room_id = "unknown"))]
async fn open_signaling_socket<B: SignalingBackend + 'static>(
    State(ctx): State<B>,
    Path(token): Path<Token>,
    ws: WebSocketUpgrade,
) -> Result<Response, B::Error> {
    tracing::debug!("Received signaling connection request");

    // Verify and consume the users roomserver token
    let signaling_context = ctx.consume_token(token).await?;
    let room_id = signaling_context.room_id;

    // This refreshes the rooms idle timeout if the room exists to avoid race conditions
    ctx.ensure_room_exists(room_id).await?;

    // Ensure guests can not join the room when guest access is disabled
    // We already enforce this when a token is requested, but guest access might have been disabled
    // after the token was issued.
    if signaling_context
        .client_parameters
        .kind
        .is_guest_or_callin()
    {
        ctx.reject_if_guest_access_not_allowed(room_id).await?;
    }

    let span = Span::current();
    span.record("opentalk.room_id", room_id.to_string());

    let response = ws.on_upgrade(move |socket| {
        handle_socket(socket, ctx, room_id, signaling_context.client_parameters).instrument(span)
    });

    Ok(response)
}

async fn handle_socket<B: SignalingBackend + 'static>(
    socket: WebSocket,
    ctx: B,
    room_id: RoomId,
    client_parameters: ClientParameters,
) {
    tracing::debug!("Upgrade to websocket connection");
    if let Err(e) = ctx
        .accept_client_stream(socket, room_id, client_parameters)
        .await
    {
        tracing::info!("Failed to accept client stream: {e:?}");
    }
}

/// Logic for handling signaling steams.
///
/// Before a new stream can be added, the WebAPI ensures that the room to which
/// the signaling stream should be added exists using
/// [`ensure_room_exists`](SignalingBackend::ensure_room_exists).
#[async_trait]
pub trait SignalingBackend: Clone + Send + Sync + std::fmt::Debug {
    type Error: Debug + IntoResponse;

    /// Returns an error if the room doesn't exist.
    async fn ensure_room_exists(&self, room_id: RoomId) -> Result<(), Self::Error>;

    /// Consume the given token from the internal token store
    ///
    /// Returns an error if the token is does not exist
    async fn consume_token(&self, token: Token) -> Result<SignalingClientContext, Self::Error>;

    /// Returns an error if guest access is disabled for the room.
    async fn reject_if_guest_access_not_allowed(&self, room_id: RoomId) -> Result<(), Self::Error>;

    /// Resolve the [`RoomId`] from the token without consuming it.
    ///
    /// Returns [`None`] if the token does not exist.
    async fn room_id(&self, token: Token) -> Option<RoomId>;

    async fn allowed_origins(&self, room_id: RoomId) -> Result<Vec<String>, Self::Error>;

    /// Accept a client stream and connect it to the room.
    ///
    /// # Error
    ///
    /// If this function errors, no response will be send to the user. The
    /// implementation must ensure to close the socket.
    async fn accept_client_stream(
        &self,
        socket: WebSocket,
        room_id: RoomId,
        client_parameters: ClientParameters,
    ) -> Result<(), Self::Error>;
}

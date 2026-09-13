// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use async_trait::async_trait;
use axum::extract::ws::{CloseFrame, WebSocket, close_code};
use opentalk_roomserver_room::Request;
use opentalk_roomserver_types::{
    client_parameters::ClientParameters, signaling::signaling_context::SignalingClientContext,
};
use opentalk_roomserver_web_api::v1::signaling::SignalingBackend;
use opentalk_types_api_internal::error::ApiError;
use opentalk_types_common::{rooms::RoomId, roomserver::Token};

use super::Context;
use crate::api::websocket::WebSocketAdapter;

#[async_trait]
impl SignalingBackend for Context {
    type Error = ApiError;

    async fn ensure_room_exists(&self, room_id: RoomId) -> Result<(), Self::Error> {
        if self.room_tasks.ensure_room_exists(&room_id).await {
            Ok(())
        } else {
            Err(ApiError::not_found())
        }
    }

    async fn consume_token(&self, token: Token) -> Result<SignalingClientContext, Self::Error> {
        self.token_store
            .lock()
            .await
            .consume_token(&token)
            .ok_or_else(|| ApiError::unauthorized().with_code("invalid_token"))
    }

    async fn reject_if_guest_access_not_allowed(&self, room_id: RoomId) -> Result<(), Self::Error> {
        match self.room_tasks.is_guest_access_allowed(room_id).await {
            Some(true) => Ok(()),
            Some(false) => Err(ApiError::forbidden()
                .with_code("guest_access_disabled")
                .with_message("Guest access disabled")),
            None => Err(ApiError::not_found()),
        }
    }

    async fn room_id(&self, token: Token) -> Option<RoomId> {
        self.token_store
            .lock()
            .await
            .peek_token(&token)
            .map(|ctx| ctx.room_id)
    }

    async fn allowed_origins(&self, room_id: RoomId) -> Result<Vec<String>, Self::Error> {
        self.room_tasks
            .allowed_origins(room_id)
            .await
            .ok_or_else(|| ApiError::not_found().with_code("invalid_room"))
    }

    async fn accept_client_stream(
        &self,
        socket: WebSocket,
        room_id: RoomId,
        client_parameters: ClientParameters,
    ) -> Result<(), Self::Error> {
        let Some(task_handle) = self.room_tasks.get_task_handle(&room_id).await else {
            error_close_websocket(socket).await;
            return Err(ApiError::not_found());
        };

        let mut res = task_handle
            .accept_signaling_socket(WebSocketAdapter::new(socket), client_parameters)
            .await;

        // handle that the socket might not reach the room task. In that case we need to close it
        // ourself.
        if let Err(e) = &mut res
            && let Some(Request::WsJoin { socket, .. }) = e.take_request()
        {
            error_close_websocket(socket.into()).await;
            return Err(ApiError::not_found());
        }

        res?;
        Ok(())
    }
}

/// Closes the websocket because of an unexpected server error.
async fn error_close_websocket(mut socket: WebSocket) {
    // the connection is probably already closed if this `send` fails
    let _ = socket
        .send(axum::extract::ws::Message::Close(Some(CloseFrame {
            code: close_code::ERROR,
            reason: "The room became unavailable.".into(),
        })))
        .await;
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use axum::{
    Router,
    extract::{Path, State, WebSocketUpgrade},
    response::Response,
    routing::any,
};
use opentalk_orchestrator_shared::services::ServiceResource;
use opentalk_types_api_common::error::ApiError;
use opentalk_types_common::roomserver::Token;

use crate::{AppState, proxy};

pub fn routes() -> Router<AppState> {
    Router::new().route("/signaling/{token}", any(roomserver_signaling))
}

pub async fn roomserver_signaling(
    Path(token): Path<Token>,
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    let room = match state.storage.consume_roomserver_token(&token).await {
        Ok(Some(room)) => room,
        Ok(None) => return Err(ApiError::forbidden().with_message("invalid token")),
        Err(e) => {
            tracing::error!("Failed to consume roomserver token: {e:?}");
            return Err(ApiError::internal().with_message("failed to consume roomserver token"));
        }
    };

    let Some((url, _)) = state
        .storage
        .get_instances_for_resource(&ServiceResource::Roomserver(room))
        .await
        .map_err(|e| {
            tracing::error!("Failed to get roomserver instance: {e:?}");
            ApiError::internal().with_message("Failed to get roomserver instance for room")
        })?
    else {
        tracing::error!(
            "Received valid roomserver token {token} but could not find associated roomserver for room {room}"
        );

        return Err(ApiError::internal());
    };

    let signaling_path = format!("/v1/signaling/{token}");
    let mut signaling_url = match url.join(&signaling_path) {
        Ok(url) => url,
        Err(e) => {
            tracing::error!("failed to build roomserver signaling url: {e}");
            return Err(
                ApiError::internal().with_message("failed to build roomserver signaling url")
            );
        }
    };

    match signaling_url.scheme() {
        "https" => signaling_url
            .set_scheme("wss")
            .expect("wss is a valid scheme"),
        "http" => signaling_url
            .set_scheme("ws")
            .expect("ws is a valid scheme"),
        scheme => {
            tracing::error!("Invalid scheme in roomserver signaling url, {scheme:?}");
            return Err(ApiError::internal());
        }
    }

    let (roomserver, _) = tokio_tungstenite::connect_async(signaling_url.as_str())
        .await
        .map_err(|e| {
            tracing::error!("failed to connect to roomserver signaling: {e}");

            ApiError::internal()
                .with_message(format!("failed to connect to roomserver instance: {e}"))
        })?;

    let response = ws.on_upgrade(|client| proxy::proxy_signaling_task(client, roomserver));

    Ok(response)
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Context;
use axum::{
    Router,
    body::Body,
    extract::{Query, RawQuery, State, WebSocketUpgrade},
    http::HeaderMap,
    response::{IntoResponse, Response as AxumResponse, Response},
    routing::{any, get},
};
use bytes::Bytes;
use opentalk_orchestrator_shared::services::ServiceResource;
use opentalk_types_api_common::error::ApiError;
use opentalk_types_common::rooms::RoomId;
use reqwest::header::AUTHORIZATION;
use serde::Deserialize;

use crate::{AppState, proxy};

#[derive(Debug, Deserialize)]
struct LivekitQuery {
    access_token: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/rtc/v1/validate", get(validate))
        .route("/rtc/validate", get(validate))
        .route("/rtc/v1", any(signaling))
        .route("/rtc", any(signaling))
}

async fn validate(
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
    Query(livekit_query): Query<LivekitQuery>,
    State(state): State<AppState>,
    body: Bytes,
) -> axum::response::Response {
    let room_id =
        match extract_room_id_from_request(livekit_query.access_token.as_deref(), &headers) {
            Ok(room_id) => room_id,
            Err(e) => {
                return e.into_response();
            }
        };

    match forward_validate_request(room_id, headers, raw_query.as_deref(), body, &state).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("Failed to forward livekit validate request to {room_id}: {e:?}");
            ApiError::internal().into_response()
        }
    }
}

async fn forward_validate_request(
    room_id: RoomId,
    headers: HeaderMap,
    raw_query: Option<&str>,
    body: Bytes,
    app_state: &AppState,
) -> anyhow::Result<AxumResponse> {
    let service_resource = ServiceResource::Roomserver(room_id);

    let (url, _) = app_state
        .storage
        .get_instances_for_resource(&service_resource)
        .await
        .context("Could not find associated instances for resource")?
        .context("Failed to select roomserver for valid livekit token")?;

    let mut url = url
        .join("livekit/rtc/validate")
        .context("Failed to build livekit validate url")?;

    tracing::debug!(
        "Forwarding livekit validate request for room {room_id} to roomserver at {url}"
    );

    url.set_query(raw_query);

    let response = app_state
        .client
        .get(url)
        .headers(headers)
        .body(body)
        .send()
        .await
        .context("Failed to send validate request to roomserver")?;

    reqwest_response_to_axum(response)
        .await
        .context("Failed to convert roomserver response to axum response")
}

async fn reqwest_response_to_axum(response: reqwest::Response) -> anyhow::Result<AxumResponse> {
    let mut axum = AxumResponse::builder().status(response.status());
    let axum_headers = axum
        .headers_mut()
        .context("Failed to to build axum response headers")?;

    for (name, value) in response.headers() {
        axum_headers.insert(name, value.clone());
    }

    let body = Body::from_stream(response.bytes_stream());

    axum.body(body).context("Failed to build axum response")
}

async fn signaling(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
    Query(livekit_query): Query<LivekitQuery>,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    let room_id = extract_room_id_from_request(livekit_query.access_token.as_deref(), &headers)?;
    let service_resource = ServiceResource::Roomserver(room_id);

    let (mut url, _) = state
        .storage
        .get_instances_for_resource(&service_resource)
        .await
        .map_err(|e| {
            tracing::error!("Failed to select instance for resource: {e:?}");
            ApiError::internal().with_message("Failed to select instance for resource")
        })?
        .ok_or_else(|| {
            tracing::error!(
                "Received known roomserver token but could not find associated roomserver for room {room_id}"
            );

            ApiError::internal()
        })?;

    match url.scheme() {
        "https" => url.set_scheme("wss").expect("wss is a valid scheme"),
        "http" => url.set_scheme("ws").expect("ws is a valid scheme"),
        scheme => {
            tracing::error!("Invalid scheme in roomserver url, {scheme:?}");
            return Err(ApiError::internal());
        }
    }

    let mut url = url.join("livekit/rtc").map_err(|e| {
        tracing::error!("failed to build livekit url: {e}");
        ApiError::internal()
    })?;

    tracing::debug!("Forwarding livekit signaling request for room {room_id} to roomserver {url}");

    url.set_query(raw_query.as_deref());

    let (roomserver_livekit_proxy, _) = tokio_tungstenite::connect_async(url.as_str())
        .await
        .map_err(|e| {
            tracing::error!("failed to connect to roomserver livekit proxy at {url}: {e}",);

            ApiError::internal()
                .with_message(format!("failed to connect to roomserver instance: {e}"))
        })?;

    let response =
        ws.on_upgrade(|client| proxy::proxy_signaling_task(client, roomserver_livekit_proxy));

    Ok(response)
}

fn extract_room_id_from_request(
    query_token: Option<&str>,
    headers: &HeaderMap,
) -> Result<RoomId, ApiError> {
    let token = match query_token {
        Some(token) => token,
        None => get_bearer_token_from_headers(headers)?,
    };

    tracing::trace!("Got livekit token {token}");

    let claims = livekit_api::access_token::Claims::from_unverified(token).map_err(|e| {
        tracing::error!("Failed to parse livekit token claims: {e}");
        ApiError::forbidden().with_message("invalid token")
    })?;

    parse_livekit_room(claims.video.room)
}

fn get_bearer_token_from_headers(headers: &HeaderMap) -> Result<&str, ApiError> {
    let Some(auth_header) = headers.get(AUTHORIZATION) else {
        return Err(ApiError::forbidden().with_message("missing token"));
    };

    let auth_header = auth_header.to_str().map_err(|e| {
        tracing::debug!("failed to parse authorization header: {e}");
        ApiError::bad_request().with_message("invalid token header")
    })?;

    let (scheme, token) = auth_header
        .split_once(' ')
        .ok_or_else(|| ApiError::bad_request().with_message("invalid auth header format"))?;

    if scheme.eq_ignore_ascii_case("bearer") {
        Ok(token)
    } else {
        Err(ApiError::forbidden().with_message("invalid token scheme"))
    }
}

/// Extract the room id from the livekit room string
///
/// Assumes the room to have the format `<room_id>[:<breakout_room_id>][#<whisper_id>]`
fn parse_livekit_room(room: String) -> Result<RoomId, ApiError> {
    tracing::trace!("Parsing room id from livekit room {room}");

    // Split off the whisper id if it exists, we don't need it for routing
    let room = if let Some((room, _whisper)) = room.split_once('#') {
        room
    } else {
        room.as_str()
    };

    // Split off the breakout room id if it exists, we don't need it for routing
    let room_id = if let Some((room_id, _breakout_room)) = room.split_once(':') {
        room_id
    } else {
        room
    };

    tracing::trace!("Got room id {room} from livekit room");

    room_id
        .parse()
        .map_err(|_| ApiError::bad_request().with_message("invalid room id"))
}

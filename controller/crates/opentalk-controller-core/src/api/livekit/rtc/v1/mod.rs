// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::sync::Arc;

use actix_web::{
    HttpRequest, HttpResponse, get,
    web::{Data, Header, Payload, Query},
};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use opentalk_controller_service::controller_backend::roomserver::SignalingProxyBackend;
use opentalk_roomserver_web_api::livekit_proxy::LiveKitQuery;
use opentalk_types_api_v1::error::ApiError;
use tokio::sync::broadcast;

use super::proxy_socket;

pub mod validate;

#[utoipa::path(
    get,
    path = "/livekit/rtc/v1",
    operation_id = "livekit_rtc_v1",
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
#[get("rtc/v1")]
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

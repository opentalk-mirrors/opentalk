// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::sync::Arc;

use actix_web::{
    HttpRequest, HttpResponse, get,
    web::{Data, Header, Query},
};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use opentalk_controller_service::controller_backend::roomserver::SignalingProxyBackend;
use opentalk_roomserver_web_api::livekit_proxy::LiveKitQuery;
use opentalk_types_api_v1::error::ApiError;

use crate::api::livekit::rtc::validate::validate;

/// Proxies the LiveKit validate request to the upstream livekit service via the room task
#[utoipa::path(
    get,
    path = "/livekit/rtc/v1/validate",
    operation_id = "livekit_rtc_v1_validate",
    responses(
        (status = StatusCode::OK, description = "Validation response from LiveKit"),
        (status = StatusCode::UNPROCESSABLE_ENTITY, description = "No livekit module configured for the request"),
        (status = StatusCode::BAD_REQUEST, description = "Invalid request headers"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "An internal server error occurred"),
    ),
    security(
        ("Livekit-Token" = [])
    ),
)]
#[tracing::instrument(level = "info", name = "/livekit/rtc/validate", skip_all)]
#[get("rtc/v1/validate")]
pub async fn get(
    proxy: Data<Option<Arc<dyn SignalingProxyBackend>>>,
    req: HttpRequest,
    auth_header: Option<Header<Authorization<Bearer>>>,
    query: Query<LiveKitQuery>,
) -> Result<HttpResponse, ApiError> {
    validate(proxy, req, auth_header, query, true).await
}

// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{str::FromStr as _, sync::Arc};

use actix_http::{
    StatusCode,
    header::{HeaderName, HeaderValue},
};
use actix_web::{
    HttpRequest, HttpResponse, get,
    web::{Data, Header, Query},
};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use livekit_api::access_token::Claims;
use opentalk_controller_service::controller_backend::roomserver::SignalingProxyBackend;
use opentalk_roomserver_web_api::livekit_proxy::{self, LiveKitQuery};
use opentalk_types_api_v1::error::ApiError;

use crate::api::livekit::rtc::{extract_access_token, raw_query, to_http1_headers};

/// Proxies the LiveKit validate request to the upstream livekit service via the room task
///
/// # Available paths
/// - `/livekit/rtc/validate`
/// - `/livekit/rtc/v1/validate`
#[utoipa::path(
    get,
    path = "/livekit/rtc/validate",
    operation_id = "livekit_rtc_validate",
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
#[get("rtc/validate")]
pub async fn get(
    proxy: Data<Option<Arc<dyn SignalingProxyBackend>>>,
    req: HttpRequest,
    auth_header: Option<Header<Authorization<Bearer>>>,
    query: Query<LiveKitQuery>,
) -> Result<HttpResponse, ApiError> {
    validate(proxy, req, auth_header, query, false).await
}

pub(crate) async fn validate(
    proxy: Data<Option<Arc<dyn SignalingProxyBackend>>>,
    req: HttpRequest,
    auth_header: Option<Header<Authorization<Bearer>>>,
    query: Query<LiveKitQuery>,
    v1: bool,
) -> Result<HttpResponse, ApiError> {
    let proxy = proxy
        .as_ref() // Option as ref
        .as_ref() // Arc as ref
        .ok_or_else(|| ApiError::not_found().with_message("LiveKit proxy not available"))?;

    let access_token = extract_access_token(query, auth_header)?;
    let content = jsonwebtoken::dangerous::insecure_decode::<Claims>(access_token.as_bytes())
        .map_err(|err| {
            tracing::debug!("Failed to decode livekit token: {err}");
            ApiError::bad_request()
        })?;

    let (room_id, _) = livekit_proxy::parse_livekit_room_id(&content.claims.video.room)?;

    let raw_query = raw_query(&req);
    let headers = to_http1_headers(req.headers());

    let response = proxy
        .proxy_livekit_validate(room_id, headers, raw_query, v1)
        .await?;

    tracing::trace!("Received validate response: {response:?}");

    let status = StatusCode::from_u16(response.status().as_u16()).map_err(|err| {
        tracing::error!("Failed to convert status code: {err}");
        ApiError::internal()
    })?;

    let mut builder = HttpResponse::build(status);

    for (name, value) in response
        .headers()
        .iter()
        .filter_map(|(name, value)| to_actix_header(name, value))
    {
        builder.insert_header((name, value));
    }

    let body = response.bytes().await.map_err(|err| {
        tracing::error!("Failed to convert response body: {err}");
        ApiError::internal()
    })?;

    Ok(builder.body(body))
}

fn to_actix_header(
    name: &http::HeaderName,
    value: &http::HeaderValue,
) -> Option<(HeaderName, HeaderValue)> {
    let name = HeaderName::from_str(name.as_str()).ok()?;
    let value = HeaderValue::from_bytes(value.as_bytes()).ok()?;

    Some((name, value))
}

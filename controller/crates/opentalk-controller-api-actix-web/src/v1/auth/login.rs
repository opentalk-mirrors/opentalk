// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/auth/login`

#![allow(deprecated)]
use actix_web::{
    get, post,
    web::{Data, Json},
};
use opentalk_controller_service_facade::OpenTalkControllerService;
use opentalk_types_api_v1::{
    auth::{GetLoginResponseBody, PostLoginResponseBody, login::AuthLoginPostRequestBody},
    error::{ApiError, AuthenticationError, ErrorBody},
};

use crate::utoipa::responses::InternalServerError;

/// **Deprecated**: This endpoint exists only for backwards compatibility and must no longer be used.
///
/// The login endpoint
///
/// Attempt to authenticate with a provided ID token. The ID token can be
/// received from an OIDC provider and contains information about the requesting
/// user as well as an expiration timestamp. When a valid token with an unknown user
/// is provided, a new user will be created in the database.
#[utoipa::path(
    request_body = AuthLoginPostRequestBody,
    tag = "api::v1::auth",
    operation_id = "post_login",
    responses(
        (
            status = StatusCode::OK,
            description = "Login successful, answer contains a list of permissions",
            body = PostLoginResponseBody,
            example = json!({"permissions": []})
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = "The provided ID token is malformed or contains invalid claims",
            body = ErrorBody,
            example = json!(
                ApiError::bad_request()
                    .with_code("invalid_claims")
                    .with_message("some required attributes are missing or malformed")
                    .body
            ),
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "The provided ID token is invalid",
            body = ErrorBody,
            example = json!(
                ApiError::unauthorized().with_www_authenticate(AuthenticationError::InvalidIdToken).body
            ),
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(),
)]
#[post("/auth/login")]
#[deprecated]
pub async fn post(
    service: Data<dyn OpenTalkControllerService>,
    body: Json<AuthLoginPostRequestBody>,
) -> Result<Json<PostLoginResponseBody>, ApiError> {
    Ok(Json(service.post_login(body.into_inner()).await?))
}

/// Get the configured OIDC provider
///
/// Returns the relevant information for a frontend to authenticate against the
/// configured OIDC provider for the OpenTalk service.
#[utoipa::path(
    tag = "api::v1::auth",
    operation_id = "get_login",
    responses(
        (
            status = StatusCode::OK,
            description = "Get information about the OIDC provider",
            body = GetLoginResponseBody,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(),
)]
#[get("/auth/login")]
pub async fn get(service: Data<dyn OpenTalkControllerService>) -> Json<GetLoginResponseBody> {
    Json(service.get_login().await)
}

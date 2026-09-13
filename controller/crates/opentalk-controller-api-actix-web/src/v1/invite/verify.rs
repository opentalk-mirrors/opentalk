// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/invite/verify`

use actix_web::{
    post,
    web::{Data, Json},
};
use opentalk_controller_service_facade::OpenTalkControllerService;
use opentalk_types_api_v1::{
    error::ApiError,
    rooms::by_room_id::invites::{PostInviteVerifyRequestBody, PostInviteVerifyResponseBody},
};

use crate::utoipa::responses::{InternalServerError, NotFound, Unauthorized};

/// Verify an invite code
///
/// Verifies the invite and returns the room url for the invite code
#[utoipa::path(
    request_body = PostInviteVerifyRequestBody,
    operation_id = "verify_invite_code",
    tag = "api::v1::invites",
    responses(
        (
            status = StatusCode::OK,
            description = "Invite is valid, the response body tells the room id",
            body = PostInviteVerifyResponseBody,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::NOT_FOUND,
            response = NotFound,
        ),
        (
            status = StatusCode::UNPROCESSABLE_ENTITY,
            description = "Invalid body contents received",
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(),
)]
#[post("/invite/verify")]
pub async fn post(
    service: Data<dyn OpenTalkControllerService>,
    verify_request: Json<PostInviteVerifyRequestBody>,
) -> Result<Json<PostInviteVerifyResponseBody>, ApiError> {
    let verify_response = service
        .verify_invite_code(verify_request.into_inner())
        .await?;

    Ok(Json(verify_response))
}

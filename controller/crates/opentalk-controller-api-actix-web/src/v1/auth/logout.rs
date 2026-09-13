// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/auth/logout`

use actix_web::{
    post,
    web::{Data, Form},
};
use opentalk_controller_service_facade::OpenTalkControllerService;
use opentalk_types_api_v1::{auth::PostLogoutRequestBody, error::ApiError};

use crate::response::NoContent;

/// Triggers back-channel OIDC logout
#[utoipa::path(
    operation_id = "post_logout",
    tag = "api::v1::auth",
    request_body(content = PostLogoutRequestBody, content_type = "application/x-www-form-urlencoded"),
    responses(
      (
            status = StatusCode::NO_CONTENT,
            description = "Logout successfull",
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = "Logout request invalid or logout failed",
        ),
    ),
    security(),
)]
#[post("/auth/logout")]
pub async fn post(
    service: Data<dyn OpenTalkControllerService>,
    request: Form<PostLogoutRequestBody>,
) -> Result<NoContent, ApiError> {
    service.post_logout(&request.logout_token).await?;
    Ok(NoContent)
}

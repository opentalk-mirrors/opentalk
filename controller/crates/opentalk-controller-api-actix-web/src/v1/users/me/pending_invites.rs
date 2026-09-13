// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/users/me/pending_invites`

use actix_web::{
    get,
    web::{Data, ReqData},
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::{error::ApiError, users::GetEventInvitesPendingResponseBody};

use crate::{
    utoipa::responses::{InternalServerError, NotFound, Unauthorized},
    v1::response::ApiResponse,
};

/// Get information about pending invites
///
/// Returns information about pending invites for the current user
#[utoipa::path(
    operation_id = "get_event_invites_pending",
    tag = "api::v1::events::invites",
    responses(
        (
            status = StatusCode::OK,
            description = "Information about pending invites is returned",
            body = GetEventInvitesPendingResponseBody,
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
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[get("/users/me/pending_invites")]
pub async fn get(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
) -> Result<ApiResponse<GetEventInvitesPendingResponseBody>, ApiError> {
    let response = service.get_event_invites_pending(current_user.id).await?;

    Ok(ApiResponse::new(response))
}

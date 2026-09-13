// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/services/call_in/start_roomserver`

use actix_web::{
    post,
    web::{Data, Json},
};
use opentalk_controller_api_actix_web::{
    host::Host,
    utoipa::responses::{InternalServerError, Unauthorized},
};
use opentalk_controller_service_facade::OpenTalkControllerService;
use opentalk_types_api_internal::call_in::PostCallInStartRoomServerRequestBody;
use opentalk_types_api_v1::{
    error::{ApiError, ErrorBody},
    rooms::by_room_id::RoomserverStartResponseBody,
};

/// Starts a signaling session for call-in on the roomserver
///
/// This endpoint is provided for call-in gateways to start a roomserver connection
/// for call-in participants. The participant typically has to provide the
/// credentials (id and pin) via DTMF (the number pad).
#[utoipa::path(
    context_path = "/call_in",
    request_body = PostCallInStartRoomServerRequestBody,
    responses(
        (
            status = StatusCode::OK,
            description = "The dial-in participant has successfully \
                authenticated for the room. Information needed for connecting to the signaling \
                is contained in the response",
            body = RoomserverStartResponseBody,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = "`id` and `pin` are not valid for any room.",
            body = ErrorBody,
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
#[post("/call_in/start")]
pub async fn post(
    service: Data<dyn OpenTalkControllerService>,
    request: Json<PostCallInStartRoomServerRequestBody>,
    host: Host,
) -> Result<Json<RoomserverStartResponseBody>, ApiError> {
    let response = service
        .start_call_in(request.into_inner(), host.into_inner())
        .await?;

    Ok(Json(response))
}

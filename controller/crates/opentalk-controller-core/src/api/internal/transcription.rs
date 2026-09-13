// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_web::{
    post,
    web::{Data, Json},
};
use opentalk_controller_api_actix_web::{
    host::Host,
    utoipa::responses::{InternalServerError, Unauthorized},
};
use opentalk_controller_service_facade::OpenTalkControllerService;
use opentalk_types_api_internal::recording::RecordingTarget;
use opentalk_types_api_v1::{
    error::{ApiError, ErrorBody},
    rooms::by_room_id::RoomserverStartResponseBody,
};

/// Starts a signaling session for transcription
///
/// This endpoint is provided for participation of transcription clients
/// which will join incognito and receive all the information and media streams required
/// for creating a transcription during the meeting.
#[utoipa::path(
    context_path = "/transcription",
    request_body = RecordingTarget,
    operation_id = "start_transcription",
    responses(
        (
            status = StatusCode::OK,
            description = "The transcription participant has successfully \
                authenticated for the room. Information needed for connecting to the signaling \
                is contained in the response",
            body = RoomserverStartResponseBody,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::NOT_FOUND,
            description = "Transcription has not been configured",
            body = ErrorBody,
            example = json!(ApiError::not_found().body),
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
#[post("/transcription/start")]
pub async fn post_start(
    service: Data<dyn OpenTalkControllerService>,
    body: Json<RecordingTarget>,
    host: Host,
) -> Result<Json<RoomserverStartResponseBody>, ApiError> {
    let response = service
        .start_transcription(body.into_inner(), host.into_inner())
        .await?;

    Ok(Json(response))
}

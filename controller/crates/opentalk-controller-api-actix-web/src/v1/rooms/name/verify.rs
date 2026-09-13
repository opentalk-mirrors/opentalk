// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! API endpoints under `v1/rooms/name/verify`

/// Verify a room name
///
/// Verifies a room name is valid and not already taken
use actix_web::{
    post,
    web::{Data, Json},
};
use opentalk_controller_service_facade::OpenTalkControllerService;
use opentalk_types_api_v1::{
    error::ApiError,
    rooms::name::{PostRoomNameVerifyRequestBody, PostRoomNameVerifyResponseBody},
};

use crate::utoipa::responses::{InternalServerError, Unauthorized};
#[utoipa::path(
   request_body = PostRoomNameVerifyRequestBody,
   operation_id = "verify_room_name",
   tag = "api::v1::rooms::name",
   responses(
      (
         status = StatusCode::OK,
         description = "Room name is valid, the response body tells if the name is available",
         body = PostRoomNameVerifyResponseBody,
      ),
      (
         status = StatusCode::UNAUTHORIZED,
         response = Unauthorized,
      ),
      (
         status = StatusCode::BAD_REQUEST,
         description = "The body or room name is invalid",
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
#[post("/rooms/name/verify")]
pub async fn post(
    service: Data<dyn OpenTalkControllerService>,
    verify_request: Json<PostRoomNameVerifyRequestBody>,
) -> Result<Json<PostRoomNameVerifyResponseBody>, ApiError> {
    Ok(Json(
        service
            .verify_room_name(verify_request.into_inner().name)
            .await?,
    ))
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/events/{event_id}/{user_id}`

use actix_web::{
    delete, patch,
    web::{Data, Json, Path, Query, ReqData},
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::{
    error::ApiError,
    events::{DeleteEventInvitePath, EventOptionsQuery, PatchInviteBody},
};
use opentalk_types_common::{events::EventId, users::UserId};

use crate::{
    response::NoContent,
    utoipa::responses::{Forbidden, InternalServerError, NotFound, Unauthorized},
};

/// Delete an invite from an event
///
/// This will uninvite the user from the event
#[utoipa::path(
    operation_id = "delete_invite_to_event",
    tag = "api::v1::events::invites",
    params(
        DeleteEventInvitePath,
        EventOptionsQuery,
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "The user event invitation has been deleted",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::FORBIDDEN,
            response = Forbidden,
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
#[delete("/events/{event_id}/invites/{user_id}")]
pub async fn delete(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,

    path_params: Path<DeleteEventInvitePath>,
    query: Query<EventOptionsQuery>,
) -> Result<NoContent, ApiError> {
    service
        .delete_invite_to_event(
            current_user.into_inner(),
            path_params.into_inner(),
            query.into_inner(),
        )
        .await?;

    Ok(NoContent)
}

/// Patch an event invite with the provided fields
///
/// Fields that are not provided in the request body will remain unchanged.
#[utoipa::path(
    operation_id = "update_invite_to_event",
    tag = "api::v1::events::invites",
    request_body = PatchInviteBody,
    params(
        ("event_id" = EventId, description = "The id of the event to be modified"),
        ("user_id" = UserId, description = "The id of the invited user to be modified"),
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "Invite was successfully updated",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::FORBIDDEN,
            description = r"The requesting user does not have the required permissions to update the invite.
              Only the creator of an event can update the invites.",
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
#[patch("/events/{event_id}/invites/{user_id}")]
pub async fn patch(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    path_parameters: Path<(EventId, UserId)>,
    update_invite: Json<PatchInviteBody>,
) -> Result<NoContent, ApiError> {
    service
        .update_invite_to_event(
            &current_user,
            path_parameters.0,
            path_parameters.1,
            &update_invite,
        )
        .await?;

    Ok(NoContent)
}

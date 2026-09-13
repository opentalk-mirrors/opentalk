// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/events/{event_id}/invites/email`

use actix_web::{
    delete, patch,
    web::{Data, Json, Path, Query, ReqData},
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::{
    error::ApiError,
    events::{DeleteEmailInviteBody, EventOptionsQuery, PatchEmailInviteBody},
};
use opentalk_types_common::events::EventId;

use crate::{
    response::NoContent,
    utoipa::responses::{Forbidden, InternalServerError, NotFound, Unauthorized},
};

/// Delete an invite from an event
///
/// Delete/Withdraw an event invitation using the email address as the identifier.
///
/// This will also withdraw invites from registered users if the provided email address matches theirs.
#[utoipa::path(
    operation_id = "delete_email_invite_to_event",
    tag = "api::v1::events::invites",
    request_body = DeleteEmailInviteBody,
    params(
        ("event_id" = EventId, description = "The id of the event"),
        EventOptionsQuery,
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "The email event invitation has been deleted",
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
#[delete("/events/{event_id}/invites/email")]
pub async fn delete(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    path: Path<EventId>,
    query: Query<EventOptionsQuery>,
    body: Json<DeleteEmailInviteBody>,
) -> Result<NoContent, ApiError> {
    service
        .delete_email_invite_to_event(
            current_user.into_inner(),
            path.into_inner(),
            body.into_inner().email,
            query.into_inner(),
        )
        .await?;

    Ok(NoContent)
}

/// Patch an event email invite with the provided fields
///
/// Fields that are not provided in the request body will remain unchanged.
#[utoipa::path(
    operation_id = "update_email_invite_to_event",
    tag = "api::v1::events::invites",
    request_body = PatchEmailInviteBody,
    params(
        ("event_id" = EventId, description = "The id of the event to be modified"),
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
#[patch("/events/{event_id}/invites/email")]
pub async fn patch(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    path_parameters: Path<EventId>,
    update_invite: Json<PatchEmailInviteBody>,
) -> Result<NoContent, ApiError> {
    service
        .update_email_invite_to_event(&current_user, path_parameters.into_inner(), &update_invite)
        .await?;

    Ok(NoContent)
}

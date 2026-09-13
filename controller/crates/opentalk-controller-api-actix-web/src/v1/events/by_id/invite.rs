// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/events/{event_id}/invite`

use actix_web::{
    delete, patch,
    web::{Data, Path, ReqData},
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::error::ApiError;
use opentalk_types_common::events::EventId;

use crate::{
    response::NoContent,
    utoipa::responses::{Forbidden, InternalServerError, NotFound, Unauthorized},
};

/// Accept an invite to an event
///
/// No content required, the request will accept the invitation.
#[utoipa::path(
    operation_id = "accept_event_invite",
    tag = "api::v1::events::invites",
    params(
        ("event_id" = EventId, description = "The id of the event"),
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "Invitation was accepted",
        ),
        (
            status = StatusCode::NOT_FOUND,
            response = NotFound,
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
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[patch("/events/{event_id}/invite")]
pub async fn patch(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    event_id: Path<EventId>,
) -> Result<NoContent, ApiError> {
    service
        .accept_event_invite(current_user.id, event_id.into_inner())
        .await?;

    Ok(NoContent)
}

/// Decline an invite to an event
///
/// No content required, the request will accept the invitation.
#[utoipa::path(
    operation_id = "decline_event_invite",
    tag = "api::v1::events::invites",
    params(
        ("event_id" = EventId, description = "The id of the event"),
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "Invitation was declined",
        ),
        (
            status = StatusCode::NOT_FOUND,
            response = NotFound,
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
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[delete("/events/{event_id}/invite")]
pub async fn delete(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    event_id: Path<EventId>,
) -> Result<NoContent, ApiError> {
    service
        .decline_event_invite(current_user.id, event_id.into_inner())
        .await?;

    Ok(NoContent)
}

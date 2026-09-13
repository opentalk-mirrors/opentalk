// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/events/{event_id}/invites`

use actix_web::{
    Either, get, post,
    web::{Data, Json, Path, Query, ReqData},
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::{
    error::ApiError,
    events::{
        EventInvitee, PostEventInviteBody, PostEventInviteQuery,
        by_event_id::invites::GetEventsInvitesQuery,
    },
};
use opentalk_types_common::events::EventId;

use crate::{
    response::{Created, NoContent},
    utoipa::responses::{BadRequest, Forbidden, InternalServerError, NotFound, Unauthorized},
    v1::response::{ApiResponse, headers::CursorLink},
};

pub mod by_id;
pub mod email;

/// Create a new invite to an event
///
/// Create a new invite to an event with the fields sent in the body.
#[utoipa::path(
    operation_id = "create_invite_to_event",
    tag = "api::v1::events::invites",
    params(
        PostEventInviteQuery,
        ("event_id" = EventId, description = "The id of the event"),
    ),
    request_body = PostEventInviteBody,
    responses(
        (
            status = StatusCode::CREATED,
            description = "The user or email has been invited to the event",
        ),
        (
            status = StatusCode::NO_CONTENT,
            description = "The user or email was already invited before, or the user is the creator of the event, in which case they have been invited implicitly",
        ),
        (
            status = StatusCode::BAD_REQUEST,
            response = BadRequest,
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
#[post("/events/{event_id}/invites")]
pub async fn post(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    event_id: Path<EventId>,
    query: Query<PostEventInviteQuery>,
    create_invite: Json<PostEventInviteBody>,
) -> Result<Either<Created, NoContent>, ApiError> {
    let created = service
        .create_invite_to_event(
            current_user.into_inner(),
            event_id.into_inner(),
            query.into_inner(),
            create_invite.into_inner(),
        )
        .await?;

    match created {
        true => Ok(Either::Left(Created)),
        false => Ok(Either::Right(NoContent)),
    }
}

/// Get the invites for an event
///
/// Returns the list of event invites
#[utoipa::path(
    operation_id = "get_invites_for_event",
    tag = "api::v1::events::invites",
    params(
        GetEventsInvitesQuery,
        ("event_id" = EventId, description = "The id of the event"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "Event invite successfully returned",
            body = EventInvitee,
            headers(
                (
                    "link" = CursorLink,
                    description = "Links for paging through the results"
                ),
            ),
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
#[get("/events/{event_id}/invites")]
pub async fn get(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    event_id: Path<EventId>,
    query: Query<GetEventsInvitesQuery>,
) -> Result<ApiResponse<Vec<EventInvitee>>, ApiError> {
    let (invitees, per_page, page, total) = service
        .get_invites_for_event(
            current_user.into_inner(),
            event_id.into_inner(),
            query.into_inner(),
        )
        .await?;

    Ok(ApiResponse::new(invitees).with_page_pagination(per_page, page, total))
}

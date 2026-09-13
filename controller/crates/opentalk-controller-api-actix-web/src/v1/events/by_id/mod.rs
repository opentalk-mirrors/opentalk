// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/events/{event_id}`

use actix_web::{
    Either, delete, get, patch,
    web::{Data, Json, Path, Query, ReqData},
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::{
    error::ApiError,
    events::{DeleteEventsQuery, EventResource, GetEventQuery, PatchEventBody, PatchEventQuery},
};
use opentalk_types_common::events::EventId;

use crate::{
    response::NoContent,
    utoipa::responses::{Forbidden, InternalServerError, NotFound, Unauthorized},
    v1::response::ApiResponse,
};

pub mod instances;
pub mod invite;
pub mod invites;
pub mod shared_folder;

/// Get an event
///
/// Returns the event resource for the given id
#[utoipa::path(
    operation_id = "get_event",
    tag = "api::v1::events",
    params(
        GetEventQuery,
        ("event_id" = EventId, description = "The id of the event"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "Event was successfully retrieved",
            body = EventResource
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
#[get("/events/{event_id}")]
pub async fn get(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    event_id: Path<EventId>,
    query: Query<GetEventQuery>,
) -> Result<Json<EventResource>, ApiError> {
    let event_resource = service
        .get_event(
            current_user.into_inner(),
            event_id.into_inner(),
            query.into_inner(),
        )
        .await?;

    Ok(Json(event_resource))
}

/// Patch an event
///
/// Fields that are not provided in the request body will remain unchanged.
#[utoipa::path(
    request_body = PatchEventBody,
    operation_id = "patch_event",
    tag = "api::v1::events",
    params(
        PatchEventQuery,
        ("event_id" = EventId, description = "The id of the event"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "The event was successfully updated",
            body = EventResource
        ),
        (
            status = StatusCode::NO_CONTENT,
            description = "The patch was empty",
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = r"Could not modify the specified event due to wrong
                syntax or bad values, for example an invalid timestamp string",
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
#[patch("/events/{event_id}")]
pub async fn patch(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    event_id: Path<EventId>,
    query: Query<PatchEventQuery>,
    patch: Json<PatchEventBody>,
) -> Result<Either<ApiResponse<EventResource>, NoContent>, ApiError> {
    let event_resource = service
        .patch_event(
            current_user.into_inner(),
            event_id.into_inner(),
            query.into_inner(),
            patch.into_inner(),
        )
        .await?;

    match event_resource {
        Some(event_resource) => Ok(Either::Left(ApiResponse::new(event_resource))),
        _ => Ok(Either::Right(NoContent)),
    }
}

/// Delete an event and its owned resources, including the associated room.
///
/// Deletes the event by the id if found. See the query parameters for affecting
/// the behavior of this endpoint, such as mail notification suppression, or
/// succeding even if external resources cannot be successfully deleted.
#[utoipa::path(
    operation_id = "delete_event",
    tag = "api::v1::events",
    params(
        DeleteEventsQuery,
        ("event_id" = EventId, description = "The id of the event"),
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "The event was successfully deleted",
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
#[delete("/events/{event_id}")]
pub async fn delete(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    event_id: Path<EventId>,
    query: Query<DeleteEventsQuery>,
) -> Result<NoContent, ApiError> {
    service
        .delete_event(
            current_user.into_inner(),
            event_id.into_inner(),
            query.into_inner(),
        )
        .await?;

    Ok(NoContent)
}

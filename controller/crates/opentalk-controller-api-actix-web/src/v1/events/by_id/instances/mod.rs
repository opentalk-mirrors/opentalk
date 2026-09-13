// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/events/{event_id}/instances`

use actix_web::{
    get,
    web::{Data, Path, Query, ReqData},
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::{
    error::ApiError,
    events::{GetEventInstancesQuery, GetEventInstancesResponseBody},
};
use opentalk_types_common::events::EventId;

use crate::{
    utoipa::responses::{Forbidden, InternalServerError, NotFound, Unauthorized},
    v1::response::{ApiResponse, headers::PageLink},
};

pub mod by_id;

/// Get a list of the instances of an event
///
/// The instances are calculated based on the RRULE of the event. If no RRULE is
/// set for the event, the single event instance is returned.
///
/// If no pagination query is added, the default page size is used.
#[utoipa::path(
    operation_id = "get_event_instances",
    tag = "api::v1::events::instances",
    params(
        GetEventInstancesQuery,
        ("event_id" = EventId, description = "The id of the event")
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "List of event instances successfully returned",
            body = GetEventInstancesResponseBody,
            headers(
                ("link" = PageLink, description = "Links for paging through the results"),
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
#[get("/events/{event_id}/instances")]
pub async fn get(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    event_id: Path<EventId>,
    query: Query<GetEventInstancesQuery>,
) -> Result<ApiResponse<GetEventInstancesResponseBody>, ApiError> {
    let (event_instances, before, after) = service
        .get_event_instances(
            &current_user.into_inner(),
            event_id.into_inner(),
            query.into_inner(),
        )
        .await?;

    Ok(ApiResponse::new(event_instances).with_cursor_pagination(before, after))
}

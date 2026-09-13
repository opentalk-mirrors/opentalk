// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/events/instances`

use actix_web::{
    get,
    web::{Data, Query, ReqData},
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::{
    error::ApiError,
    events::{EventOrInstance, GetEventInstancesResponseBody, GetEventsAndInstancesQuery},
};

use crate::{
    utoipa::responses::{Forbidden, InternalServerError, NotFound, Unauthorized},
    v1::response::{ApiResponse, headers::PageLink},
};

/// Get a list of events and instances
///
/// The instances are calculated based on the RRULE of the event. If no RRULE is
/// set for an event, the event itself is returned
///
/// Returns a paginated list of events or their instances inside the given time range
#[utoipa::path(
    operation_id = "get_events_and_instances",
    tag = "api::v1::events::instances",
    params(
        GetEventsAndInstancesQuery,
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "List of events and instances successfully returned",
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
#[get("/events/instances")]
pub async fn get(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    query: Query<GetEventsAndInstancesQuery>,
) -> Result<ApiResponse<Vec<EventOrInstance>>, ApiError> {
    let (resources, before, after) = service
        .get_events_and_instances_interwoven(current_user.into_inner(), query.into_inner())
        .await?;

    Ok(ApiResponse::new(resources).with_cursor_pagination(before, after))
}

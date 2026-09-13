// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::fmt::Debug;

use async_trait::async_trait;
use axum::{
    Json,
    extract::{Path, State},
    routing::post,
};
use opentalk_types_api_internal::{error::ApiError, module_assets::Quota};
use opentalk_types_common::users::UserId;

use crate::Router;

pub fn routes<B: UserBackend + 'static>() -> Router<B> {
    Router::new().nest(
        "/user",
        Router::new().route("/{user_id}/storage_quota", post(post_storage_quota::<B>)),
    )
}

#[async_trait]
pub trait UserBackend: Clone + Send + Sync + Debug {
    async fn post_storage_quota(&self, user_id: UserId, quota: Quota) -> Result<(), ApiError>;
}

/// Updates the storage quota (used and limit) of the specified user.
#[utoipa::path(
    post,
    path = "/user/{user_id}/storage_quota",
    request_body = Quota,
    params(
        ("user_id" = UserId, Path, description = "The UUID that identifies the user")
    ),
    responses(
        (status = StatusCode::NO_CONTENT, description = "The storage quota was updated successfully"),
        (status = StatusCode::NOT_FOUND, description = "No rooms created by the specified user exist"),
    ),
    security(
        ("API-Token" = [])
    )
)]
pub(crate) async fn post_storage_quota<B: UserBackend>(
    State(ctx): State<B>,
    Path(user_id): Path<UserId>,
    Json(quota): Json<Quota>,
) -> Result<(), ApiError> {
    ctx.post_storage_quota(user_id, quota).await
}

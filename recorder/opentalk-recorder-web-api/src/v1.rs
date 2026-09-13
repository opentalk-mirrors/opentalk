// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
use async_trait::async_trait;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json,
};
use opentalk_types_api_common::error::ApiError;
use opentalk_types_api_internal::recording::RecordingTarget;
use serde::{Deserialize, Serialize};

use super::Router;

pub trait Backend: Send + Sync + Clone + Sized {}

const API_VERSION: &str = "/v1";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum RecordingAction {
    Created,
    AlreadyRunning,
}

impl IntoResponse for RecordingAction {
    fn into_response(self) -> Response {
        match self {
            Self::Created => StatusCode::CREATED.into_response(),
            Self::AlreadyRunning => StatusCode::NO_CONTENT.into_response(),
        }
    }
}

#[async_trait]
pub trait RecorderBackend: Clone + Send + Sync {
    async fn init(&self, recording: RecordingTarget) -> Result<RecordingAction, ApiError>;
}

async fn init<B: RecorderBackend>(
    State(ctx): State<B>,
    Json(recording): Json<RecordingTarget>,
) -> Result<RecordingAction, ApiError> {
    ctx.init(recording).await
}

pub fn routes<B: RecorderBackend + 'static>() -> Router<B> {
    Router::<B>::new().nest(API_VERSION, Router::new().route("/init", post(init::<B>)))
}

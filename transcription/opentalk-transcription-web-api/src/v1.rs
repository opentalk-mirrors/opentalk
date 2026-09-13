// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use async_trait::async_trait;
use axum::{Json, extract::State, routing::post};
use opentalk_types_api_common::error::ApiError;
use opentalk_types_common::rooms::RoomId;
use serde::{Deserialize, Serialize};

use super::Router;

pub trait Backend: Send + Sync + Clone + Sized {}

const API_VERSION: &str = "/v1";

/// Request Body used by transcription for the `POST /v1/init`
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct TranscriptionTarget {
    /// The room id which the transcription service shall join to
    pub room_id: RoomId,

    /// The optional breakout room id in which the transcription was requested
    ///
    /// This is the internal equivalent to opentalk-roomserver-types's `BreakoutId`
    pub breakout_room: Option<u32>,

    /// Target language of the transcription, specified as a two-letter language code (e.g., `de`,
    /// `en`).
    ///
    /// Set to `None` for auto-detection.
    pub language: Option<String>,
}

#[async_trait]
pub trait TranscriptionBackend: Clone + Send + Sync {
    async fn init(&self, room_info: TranscriptionTarget) -> Result<(), ApiError>;
}

async fn init<B: TranscriptionBackend>(
    State(ctx): State<B>,
    Json(room_info): Json<TranscriptionTarget>,
) -> Result<(), ApiError> {
    ctx.init(room_info).await
}

pub fn routes<B: TranscriptionBackend + 'static>() -> Router<B> {
    Router::<B>::new().nest(API_VERSION, Router::new().route("/init", post(init::<B>)))
}

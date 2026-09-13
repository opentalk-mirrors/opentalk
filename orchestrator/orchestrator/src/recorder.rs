// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_orchestrator_shared::RecorderResource;
use opentalk_recorder_web_api::v1::{RecorderBackend, RecordingAction};
use opentalk_types_api_common::error::{ApiError, ErrorBody};
use opentalk_types_api_internal::recording::RecordingTarget;
use reqwest::{StatusCode, header::AUTHORIZATION};

use crate::{AppState, service_instance::selection::SelectedInstance};

#[async_trait::async_trait]
impl RecorderBackend for AppState {
    async fn init(&self, target: RecordingTarget) -> Result<RecordingAction, ApiError> {
        let recorder_resource = RecorderResource {
            room_id: target.room_id,
            breakout_id: target.breakout_room,
        };

        let SelectedInstance {
            address,
            auth_header,
        } = self.select_recorder(recorder_resource).await?;

        let url = address
            .join("v1/init")
            .map_err(|_| ApiError::internal().with_message("Failed to construct recorder URL"))?;

        let response = self
            .client
            .post(url)
            .header(AUTHORIZATION, auth_header)
            .json(&target)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to send request to recorder: {e}");
                ApiError::internal().with_message("Recorder unreachable")
            })?;

        let status = response.status();
        let body = response.bytes().await.map_err(|e| {
            tracing::error!("Failed to read recorder response body: {e}");
            ApiError::internal().with_message("Failed to read recorder response body")
        })?;

        match status {
            StatusCode::OK => Ok(RecordingAction::Created),
            StatusCode::CREATED => Ok(RecordingAction::Created),
            StatusCode::NO_CONTENT => Ok(RecordingAction::AlreadyRunning),
            StatusCode::UNAUTHORIZED => {
                Err(ApiError::internal().with_message("Failed to authorize at recorder"))
            }

            error_code => {
                let error_body = match serde_json::from_slice::<ErrorBody>(&body) {
                    Ok(t) => Ok(t),
                    Err(e) => {
                        tracing::error!(
                            "Unexpected recorder response body for status {status}: {e}\nBody:\n{}",
                            String::from_utf8_lossy(&body)
                        );
                        Err(ApiError::internal().with_message("Unexpected recorder response body"))
                    }
                }?;

                Err(ApiError {
                    status: error_code,
                    www_authenticate: None,
                    body: error_body,
                })
            }
        }
    }
}

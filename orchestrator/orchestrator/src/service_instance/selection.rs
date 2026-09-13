// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_orchestrator_shared::{
    RecorderResource, TranscriptionResource, services::ServiceResource,
};
use opentalk_service_auth::{ApiKeyId, EncodingError};
use opentalk_types_api_common::error::ApiError;
use opentalk_types_common::rooms::RoomId;
use url::Url;

use crate::AppState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectedInstance {
    /// The address of the selected instance
    pub(crate) address: Url,
    /// The authorization header for request towards the instance
    pub(crate) auth_header: String,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum InstanceError {
    #[error("no instance of the requested service are currently available")]
    NotAvailable,

    #[error("no matching service API keys found for key ids: {0:#?}")]
    UnknownApiKeyIds(Vec<ApiKeyId>),

    #[error("failed to create authorization header for key id {key_id}: {err:?}")]
    AuthHeader {
        key_id: ApiKeyId,
        err: EncodingError,
    },
}

impl From<InstanceError> for ApiError {
    fn from(instance_error: InstanceError) -> Self {
        match instance_error {
            InstanceError::NotAvailable => ApiError::service_unavailable()
                .with_message("not_available")
                .with_message(instance_error.to_string()),
            InstanceError::UnknownApiKeyIds { .. } | InstanceError::AuthHeader { .. } => {
                ApiError::internal().with_message("internal authorization error")
            }
        }
    }
}

impl AppState {
    pub(crate) async fn select_roomserver(
        &self,
        room_id: RoomId,
    ) -> Result<SelectedInstance, InstanceError> {
        self.select_instance(ServiceResource::Roomserver(room_id))
            .await
    }

    pub(crate) async fn select_recorder(
        &self,
        recorder_resource: RecorderResource,
    ) -> Result<SelectedInstance, InstanceError> {
        self.select_instance(ServiceResource::Recorder(recorder_resource))
            .await
    }

    pub(crate) async fn select_transcription(
        &self,
        transcription_resource: TranscriptionResource,
    ) -> Result<SelectedInstance, InstanceError> {
        self.select_instance(ServiceResource::Transcription(transcription_resource))
            .await
    }

    async fn select_instance(
        &self,
        service_resource: ServiceResource,
    ) -> Result<SelectedInstance, InstanceError> {
        let (address, instance_data) = self
            .storage
            .select_instance_for_resource(service_resource)
            .await
            .map_err(|e| {
                tracing::error!("Failed to select instance for resource {service_resource}: {e:?}");
                InstanceError::NotAvailable
            })?;

        let auth_header = self.get_auth_header_for_key_ids(&instance_data.api_key_ids)?;

        Ok(SelectedInstance {
            address: address.clone(),
            auth_header,
        })
    }

    fn get_auth_header_for_key_ids(&self, key_ids: &[ApiKeyId]) -> Result<String, InstanceError> {
        let Some(api_key) = self.get_api_key_for_key_ids(key_ids) else {
            return Err(InstanceError::UnknownApiKeyIds(key_ids.to_vec()));
        };

        let jwt = match api_key.generate_jwt() {
            Ok(jwt) => format!("Bearer {jwt}"),
            Err(err) => {
                return Err(InstanceError::AuthHeader {
                    key_id: api_key.id,
                    err,
                });
            }
        };

        Ok(jwt)
    }
}

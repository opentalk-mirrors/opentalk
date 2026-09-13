// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use opentalk_orchestrator_shared::{
    Metrics, OrchestratorMetrics, ServiceKind,
    error::RegistrationError,
    services::{InstanceData, ServiceResource},
};
use opentalk_types_common::{rooms::RoomId, roomserver::Token};
use url::Url;

use crate::service_instance::registration::ServiceRegistration;

pub(crate) mod local;
pub(crate) mod redis;
#[cfg(test)]
pub(crate) mod tests;

/// Longer default expiry than the roomserver default token expiry. This avoids having tokens expire
/// in the orchestrator before they do in the roomserver
const ROOMSERVER_TOKEN_EXPIRY: Duration = Duration::from_secs(60);

#[derive(Debug, thiserror::Error)]
pub(crate) enum AddInstanceError {
    #[error(transparent)]
    RegistrationError(#[from] RegistrationError),
    #[error("Internal storage error")]
    Internal(#[from] anyhow::Error),
}

#[async_trait::async_trait]
pub(crate) trait OrchestratorStorage: std::fmt::Debug + Sync + Send {
    /// Get the current metrics for the orchestrator
    async fn get_orchestrator_metrics(&self) -> anyhow::Result<OrchestratorMetrics>;

    async fn set_service_metrics(
        &self,
        url: &Url,
        kind: ServiceKind,
        metrics: Metrics,
    ) -> anyhow::Result<()>;

    async fn add_instance(&self, registration: ServiceRegistration)
    -> Result<(), AddInstanceError>;

    #[allow(dead_code)]
    async fn get_instance(
        &self,
        url: &Url,
        kind: ServiceKind,
    ) -> anyhow::Result<Option<InstanceData>>;

    async fn remove_instance(&self, url: &Url, kind: ServiceKind) -> anyhow::Result<()>;

    async fn remove_service_resource(
        &self,
        url: &Url,
        resource: ServiceResource,
    ) -> anyhow::Result<()>;

    async fn get_instances_for_resource(
        &self,
        resource: &ServiceResource,
    ) -> anyhow::Result<Option<(Url, InstanceData)>>;

    /// Select the associated instance for the given resource.
    ///
    /// If no related instance is found, the lowest loaded instance of the corresponding service
    /// kind is returned.
    ///
    /// If no instance of the corresponding service kind is available, an error is returned.
    async fn select_instance_for_resource(
        &self,
        resource: ServiceResource,
    ) -> anyhow::Result<(Url, InstanceData)>;

    /// Add a roomserver token to the internal token store, allowing the orchestrator to map
    /// `/rooms/start` with a known token to the associated room id
    async fn add_roomserver_token(&self, room_id: RoomId, token: Token) -> anyhow::Result<()>;

    /// Consume a known roomserver token, removing it from the internal token store
    async fn consume_roomserver_token(&self, token: &Token) -> anyhow::Result<Option<RoomId>>;

    /// Overwrite the default [`ROOMSERVER_TOKEN_EXPIRY`]
    #[cfg(test)]
    async fn set_roomserver_token_expiry(&mut self, expiry: Duration);

    #[cfg(test)]
    async fn assert_empty(&self) -> anyhow::Result<()>;

    #[cfg(test)]
    async fn get_all_resources(&self) -> anyhow::Result<Vec<String>>;

    #[cfg(test)]
    async fn get_all_services(&self) -> anyhow::Result<Vec<String>>;
}

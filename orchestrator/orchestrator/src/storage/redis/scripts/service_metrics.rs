// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashMap;

use anyhow::Context;
use opentalk_orchestrator_shared::{
    Metrics,
    services::{InstanceData, ResourceType, ServiceResource, ServiceState},
};
use opentalk_service_auth::ApiKeyId;
use redis::{FromRedisValue, Script};
use url::Url;

use crate::storage::redis::keys::{ServiceId, ServiceKindKey};

/// Raw per-service data returned by the [`SCRIPT`].
struct ServiceMetricsEntry {
    /// Base64-encoded service URL
    service_id: String,
    /// JSON-encoded `Vec<String>` of API key IDs.
    api_key_ids_json: String,
    /// Current load of the service instance
    load: u8,
    /// Whether the service instance is currently accepting jobs
    accepting_jobs: bool,
    /// Serialized [`ServiceResource`] strings.
    resources: Vec<String>,
}

impl FromRedisValue for ServiceMetricsEntry {
    fn from_redis_value(v: redis::Value) -> Result<Self, redis::ParsingError> {
        let (service_id, api_key_ids_json, load, accepting_jobs, resources) =
            <(String, String, u8, bool, Vec<String>)>::from_redis_value(v)?;
        Ok(Self {
            service_id,
            api_key_ids_json,
            load,
            accepting_jobs,
            resources,
        })
    }
}

const SCRIPT: &str = include_str!("service_metrics.lua");

/// Fetches instance data, metrics, and managed resources for every service of the provided resource
/// type
pub(crate) async fn get_service_metrics<T: ResourceType>(
    client: &redis::Client,
) -> anyhow::Result<HashMap<Url, ServiceState<T>>> {
    let kind = T::kind();

    let mut con = client
        .get_multiplexed_async_connection()
        .await
        .context("Failed to get Redis connection for service metrics")?;

    let entries = Script::new(SCRIPT)
        .key(ServiceKindKey { kind })
        .invoke_async::<Vec<ServiceMetricsEntry>>(&mut con)
        .await
        .with_context(|| format!("Failed to run service_metrics script for kind {kind:?}"))?;

    let mut service_metrics = HashMap::with_capacity(entries.len());

    for ServiceMetricsEntry {
        service_id,
        api_key_ids_json,
        load,
        accepting_jobs,
        resources: resource_strs,
    } in entries
    {
        let id = ServiceId::try_from(service_id.as_str()).context("Failed to decode service_id")?;
        let url = Url::try_from(&id)
            .with_context(|| format!("Failed to decode service URL for id '{}'", id.as_str()))?;

        let api_key_ids: Vec<ApiKeyId> = serde_json::from_str::<Vec<String>>(&api_key_ids_json)
            .context("Failed to deserialize api_key_ids")?
            .into_iter()
            .map(ApiKeyId::from)
            .collect();

        let instance_data = InstanceData {
            metrics: Metrics {
                load,
                accepting_jobs,
            },
            api_key_ids,
        };

        let managed_resources = resource_strs
            .into_iter()
            .filter_map(|s| match ServiceResource::try_from(s.as_str()) {
                Ok(resource) => match T::from_service_resource(resource) {
                    Some(r) => Some(r),
                    None => {
                        tracing::warn!("Resource '{s}' does not match expected kind {kind:?}");
                        None
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to parse resource '{s}': {e}");
                    None
                }
            })
            .collect();

        service_metrics.insert(
            url,
            ServiceState {
                instance_data,
                managed_resources,
            },
        );
    }

    Ok(service_metrics)
}

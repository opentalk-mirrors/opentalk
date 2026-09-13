// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Context;
use opentalk_orchestrator_shared::{ServiceKind, error::RegistrationError};
use opentalk_service_auth::ApiKeyId;
use redis::{FromRedisValue, RedisError, ToRedisArgs};
use url::Url;
use uuid::Uuid;

use crate::storage::{
    ServiceResource,
    redis::keys::{
        OrchestratorServicesKey, ResourceLookupKey, ServiceId, ServiceInstanceKey, ServiceKindKey,
        ServiceMetricsKey, ServiceResourceKey,
    },
};

#[derive(Debug)]
enum ScriptResult {
    Ok,
    AddressAlreadyInUse,
    ResourceConflict(Vec<ServiceResource>),
}

impl FromRedisValue for ScriptResult {
    fn from_redis_value(v: redis::Value) -> Result<Self, redis::ParsingError> {
        let (code, conflicting_resources) = <(i32, Vec<String>)>::from_redis_value(v)?;

        match code {
            0 => Ok(ScriptResult::Ok),
            1 => Ok(ScriptResult::AddressAlreadyInUse),
            2 => {
                let resources = conflicting_resources
                    .iter()
                    .map(|resource| ServiceResource::try_from(resource.as_str()))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| format!("failed to parse conflicting resource: {e}"))?;

                Ok(ScriptResult::ResourceConflict(resources))
            }
            other => Err(format!("unexpected add_instance script return value: {other}").into()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum AddInstanceScriptError {
    #[error("{0}")]
    Registration(RegistrationError),
    #[error("{0}")]
    Redis(#[from] RedisError),
    #[error("{0}")]
    Internal(#[from] anyhow::Error),
}

impl From<ScriptResult> for Result<(), AddInstanceScriptError> {
    fn from(value: ScriptResult) -> Self {
        match value {
            ScriptResult::Ok => Ok(()),
            ScriptResult::AddressAlreadyInUse => Err(AddInstanceScriptError::Registration(
                RegistrationError::AddressAlreadyInUse,
            )),
            ScriptResult::ResourceConflict(resources) => Err(AddInstanceScriptError::Registration(
                RegistrationError::ResourceConflict(resources),
            )),
        }
    }
}

const SCRIPT: &str = include_str!("add_instance.lua");

/// Add a new service instance to the orchestrator cluster.
///
/// The script will initialize all relevant keys and check for conflicts with existing instances or
/// resources.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn add_instance(
    client: &redis::Client,
    url: Url,
    kind: ServiceKind,
    key_ids: Vec<ApiKeyId>,
    load: u8,
    accepting_jobs: bool,
    managed_resources: Vec<ServiceResource>,
    orchestrator_id: Uuid,
) -> Result<(), AddInstanceScriptError> {
    let script = redis::Script::new(SCRIPT);
    let service_id = ServiceId::from(&url);

    let key_ids = serde_json::to_string(
        &key_ids
            .iter()
            .map(|key_id| key_id.to_string())
            .collect::<Vec<String>>(),
    )
    .context("Failed to serialize key ids")?;

    let mut resource_values = Vec::with_capacity(managed_resources.len());
    let mut resource_keys = Vec::with_capacity(managed_resources.len());

    for resource in &managed_resources {
        resource_values.push(resource.to_string());

        let resource_lookup_key = ResourceLookupKey {
            resource: *resource,
        };

        resource_keys.push(
            String::from_utf8_lossy(
                resource_lookup_key
                    .to_redis_args()
                    .first()
                    .context("Failed to build resource lookup key")?,
            )
            .to_string(),
        );
    }

    debug_assert_eq!(resource_keys.len(), managed_resources.len());

    let resource_keys = serde_json::to_string(&resource_keys)
        .context("Failed to serialize resource lookup keys")?;
    let resource_values =
        serde_json::to_string(&resource_values).context("Failed to serialize resource values")?;
    let mut con = client.get_multiplexed_async_connection().await?;

    let result = script
        .key(ServiceInstanceKey {
            id: service_id.clone(),
        })
        .key(ServiceMetricsKey {
            id: service_id.clone(),
        })
        .key(ServiceResourceKey {
            id: service_id.clone(),
        })
        .key(ServiceKindKey { kind })
        .key(OrchestratorServicesKey {
            id: orchestrator_id.to_string(),
        })
        .arg(service_id.as_str())
        .arg(url.as_str())
        .arg(kind.to_string())
        .arg(key_ids)
        .arg(load)
        .arg(accepting_jobs)
        .arg(resource_values)
        .arg(resource_keys)
        .invoke_async::<ScriptResult>(&mut con)
        .await?;

    result.into()
}

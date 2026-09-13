// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Context;
use opentalk_orchestrator_shared::{
    Metrics,
    services::{InstanceData, ServiceResource},
};
use opentalk_service_auth::ApiKeyId;
use redis::{FromRedisValue, Script};
use url::Url;

use crate::storage::redis::keys::{ResourceLookupKey, ServiceId};

type ReturnedInstance = (
    String,     // service_id (base64-encoded url)
    String,     // api_key_ids
    (u8, bool), // load, accepting_jobs
);

#[derive(Debug)]
enum ScriptResult {
    Ok(ReturnedInstance),
    NotFound,
}

impl FromRedisValue for ScriptResult {
    fn from_redis_value(v: redis::Value) -> Result<Self, redis::ParsingError> {
        if let Ok(returned_instance) = ReturnedInstance::from_redis_value(v.clone()) {
            return Ok(ScriptResult::Ok(returned_instance));
        }

        match i32::from_redis_value(v)? {
            1 => Ok(ScriptResult::NotFound),
            other => Err(format!(
                "unexpected get_instance_by_resource script return value: {other}"
            )
            .into()),
        }
    }
}

const SCRIPT: &str = include_str!("get_by_resource.lua");

/// Fetches the service instance that manages the given resource, if any.
pub(crate) async fn get_instance_by_resource(
    client: &redis::Client,
    service_resource: &ServiceResource,
) -> anyhow::Result<Option<(Url, InstanceData)>> {
    let script = Script::new(SCRIPT);
    let mut con = client.get_multiplexed_async_connection().await?;

    let result: ScriptResult = script
        .key(ResourceLookupKey {
            resource: *service_resource,
        })
        .invoke_async(&mut con)
        .await
        .context("Failed to run get_instance_by_resource script")?;

    let (service_id, api_key_ids_str, (load, accepting_jobs)) = match result {
        ScriptResult::NotFound => return Ok(None),
        ScriptResult::Ok(instance) => instance,
    };

    let url = (&ServiceId::try_from(service_id.as_str())?)
        .try_into()
        .context("Failed to parse url from service_id")?;

    let api_key_ids: Vec<ApiKeyId> = serde_json::from_str::<Vec<String>>(&api_key_ids_str)
        .context("Failed to deserialize api_key_ids")?
        .into_iter()
        .map(ApiKeyId::from)
        .collect();

    Ok(Some((
        url,
        InstanceData {
            metrics: Metrics {
                load,
                accepting_jobs,
            },
            api_key_ids,
        },
    )))
}

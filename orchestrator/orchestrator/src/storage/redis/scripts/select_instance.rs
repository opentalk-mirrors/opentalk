// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Context;
use base64::{Engine as _, engine::general_purpose};
use opentalk_orchestrator_shared::{Metrics, ServiceKind};
use opentalk_service_auth::ApiKeyId;
use redis::FromRedisValue;
use url::Url;

use crate::storage::{
    InstanceData, ServiceResource,
    redis::keys::{ResourceLookupKey, ServiceKindKey},
};

type ReturnedInstance = (
    String,     // url
    String,     // api key ids
    (u8, bool), // load, accepting jobs
);

#[derive(Debug)]
enum ScriptResult {
    Ok(ReturnedInstance),
    NoInstanceAvailable, // 1
}

impl FromRedisValue for ScriptResult {
    fn from_redis_value(v: redis::Value) -> Result<Self, redis::ParsingError> {
        if let Ok(returned_instance) = ReturnedInstance::from_redis_value(v.clone()) {
            return Ok(ScriptResult::Ok(returned_instance));
        }

        let int_value = i32::from_redis_value(v)?;

        match int_value {
            1 => Ok(ScriptResult::NoInstanceAvailable),
            other => Err(format!("unexpected select_instance script return value: {other}").into()),
        }
    }
}

const SCRIPT: &str = include_str!("select_instance.lua");

/// Select the service that manages the given resource.
///
/// If no service currently manages the resource, the service with the lowest load that is accepting
/// jobs will be selected and assigned to manage the resource.
pub(crate) async fn select_instance(
    client: &redis::Client,
    resource: ServiceResource,
) -> anyhow::Result<(Url, InstanceData)> {
    let script = redis::Script::new(SCRIPT);
    let mut con = client.get_multiplexed_async_connection().await?;

    let kind = match resource {
        ServiceResource::Roomserver(_) => ServiceKind::Roomserver,
        ServiceResource::Recorder(_) => ServiceKind::Recorder,
        ServiceResource::Transcription(_) => ServiceKind::Transcription,
    };

    let result: ScriptResult = script
        .key(ResourceLookupKey { resource })
        .key(ServiceKindKey { kind })
        .arg(resource.to_string())
        .invoke_async(&mut con)
        .await
        .context("Failed to select instance")?;

    let (service_id, api_key_ids, (load, accepting_jobs)) = match result {
        ScriptResult::Ok((service_id, api_key_ids, (load, accepting_jobs))) => {
            (service_id, api_key_ids, (load, accepting_jobs))
        }
        ScriptResult::NoInstanceAvailable => {
            anyhow::bail!("No instance available for resource: {resource:?}")
        }
    };

    let url_str = general_purpose::STANDARD
        .decode(service_id)
        .context("Failed to decode base64 url")?;
    let url_str = String::from_utf8_lossy(url_str.as_ref());
    let url = Url::parse(&url_str).context("Failed to parse url from decoded base64")?;

    let api_key_ids: Vec<ApiKeyId> = serde_json::from_str::<Vec<String>>(&api_key_ids)
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

    Ok((url, instance_data))
}

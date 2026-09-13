// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::anyhow;
use opentalk_orchestrator_shared::ServiceKind;
use redis::FromRedisValue;
use url::Url;
use uuid::Uuid;

use crate::storage::redis::keys::{
    OrchestratorServicesKey, ServiceId, ServiceInstanceKey, ServiceKindKey,
};

#[derive(Debug)]
enum ScriptResult {
    Ok = 0,
    NotFound = 1,
}

impl FromRedisValue for ScriptResult {
    fn from_redis_value(v: redis::Value) -> Result<Self, redis::ParsingError> {
        let int_value = i32::from_redis_value(v)?;
        match int_value {
            0 => Ok(ScriptResult::Ok),
            1 => Ok(ScriptResult::NotFound),
            other => Err(format!("unexpected remove_instance script return value: {other}").into()),
        }
    }
}

const SCRIPT: &str = include_str!("remove_instance.lua");

/// Removes a service instance and all its associated data from Redis.
pub(crate) async fn remove_instance(
    client: &redis::Client,
    orchestrator_id: Uuid,
    url: &Url,
    kind: ServiceKind,
) -> anyhow::Result<()> {
    let script = redis::Script::new(SCRIPT);
    let id = ServiceId::from(url);
    let mut con = client.get_multiplexed_async_connection().await?;

    let result: ScriptResult = script
        .key(ServiceInstanceKey { id: id.clone() })
        .key(ServiceKindKey { kind })
        .key(OrchestratorServicesKey {
            id: orchestrator_id.to_string(),
        })
        .arg(id.to_string())
        .invoke_async(&mut con)
        .await?;

    match result {
        ScriptResult::Ok => Ok(()),
        ScriptResult::NotFound => Err(anyhow!("Service instance for the given URL not found")),
    }
}

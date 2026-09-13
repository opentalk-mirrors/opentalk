// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Context;
use redis::FromRedisValue;
use uuid::Uuid;

use crate::storage::redis::keys::{
    ORCHESTRATORS_SET_KEY, OrchestratorAliveKey, OrchestratorServicesKey,
};

#[derive(Debug, PartialEq)]
enum ScriptResult {
    /// The orchestrator was dead and its state was removed.
    Ok = 0,
    /// The orchestrator's alive key was still present — no cleanup performed.
    StillAlive = 1,
    /// The orchestrator ID was not found in the global orchestrators set.
    NotFound = 2,
}

impl FromRedisValue for ScriptResult {
    fn from_redis_value(v: redis::Value) -> Result<Self, redis::ParsingError> {
        match i32::from_redis_value(v)? {
            0 => Ok(ScriptResult::Ok),
            1 => Ok(ScriptResult::StillAlive),
            2 => Ok(ScriptResult::NotFound),
            other => Err(format!("unexpected cleanup script return value: {other}").into()),
        }
    }
}

/// Atomically cleans up a single dead orchestrator.
const SCRIPT: &str = include_str!("cleanup.lua");

pub(crate) async fn cleanup(
    client: &redis::Client,
    orchestrator_ids: &[Uuid],
) -> anyhow::Result<Vec<Uuid>> {
    if orchestrator_ids.is_empty() {
        return Ok(Vec::new());
    }

    let script = redis::Script::new(SCRIPT);
    let mut con = client
        .get_multiplexed_async_connection()
        .await
        .context("Failed to get Redis connection for orchestrator cleanup")?;

    let mut cleaned_up = Vec::new();

    for id in orchestrator_ids {
        let id_str = id.clone().to_string();

        let result = script
            .key(OrchestratorAliveKey { id: id_str.clone() })
            .key(OrchestratorServicesKey { id: id_str.clone() })
            .key(ORCHESTRATORS_SET_KEY)
            .arg(id_str.clone())
            .invoke_async::<ScriptResult>(&mut con)
            .await
            .with_context(|| format!("Failed to run cleanup script for orchestrator '{id_str}'"))?;

        match result {
            ScriptResult::Ok => {
                tracing::info!("Cleaned up storage for dead orchestrator '{id_str}'");
                cleaned_up.push(*id);
            }
            ScriptResult::StillAlive => {
                tracing::debug!(
                    "Tried to clean up orchestrator '{id_str}' that is still alive, skipping cleanup"
                );
            }
            ScriptResult::NotFound => {
                // This can happen if the orchestrator was already cleaned up by another instance.
                tracing::debug!(
                    "Tried to clean up orchestrator '{id_str}' that was not found in the global set, skipping cleanup"
                );
            }
        }
    }

    Ok(cleaned_up)
}

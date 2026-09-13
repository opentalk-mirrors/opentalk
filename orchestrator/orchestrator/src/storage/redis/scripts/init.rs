// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Context;
use redis::FromRedisValue;
use uuid::Uuid;

use crate::storage::redis::{
    KEEP_ALIVE_TTL,
    keys::{ORCHESTRATORS_SET_KEY, OrchestratorAliveKey},
};

const SCRIPT: &str = include_str!("init.lua");
const UUID_COLLISION_RETRIES: u32 = 3;

/// Result of the init script.
#[derive(Debug)]
enum ScriptResult {
    /// Initialization succeeded
    Ok = 0,
    /// The generated UUID is already in use
    UuidCollision = 1,
}

impl FromRedisValue for ScriptResult {
    fn from_redis_value(v: redis::Value) -> Result<Self, redis::ParsingError> {
        match v {
            redis::Value::Int(0) => Ok(ScriptResult::Ok),
            redis::Value::Int(1) => Ok(ScriptResult::UuidCollision),
            other => Err(format!("unexpected init script return value: {other:?}").into()),
        }
    }
}

///  Registers this orchestrator with its uuid in Redis, and adds it to the global orchestrators set
pub(crate) async fn init(client: &redis::Client) -> anyhow::Result<Uuid> {
    let script = redis::Script::new(SCRIPT);
    let mut con = client
        .get_multiplexed_async_connection()
        .await
        .context("Failed to get Redis connection")?;

    let mut retries = 0u32;
    loop {
        let id = Uuid::new_v4();
        let id_str = id.to_string();

        let result = script
            .key(OrchestratorAliveKey { id: id_str.clone() })
            .key(ORCHESTRATORS_SET_KEY)
            .arg(&id_str)
            .arg(KEEP_ALIVE_TTL)
            .invoke_async::<ScriptResult>(&mut con)
            .await
            .context("Failed to execute orchestrator init script")?;

        match result {
            ScriptResult::Ok => {
                return Ok(id);
            }
            ScriptResult::UuidCollision => {
                retries += 1;
                if retries > UUID_COLLISION_RETRIES {
                    return Err(anyhow::anyhow!(
                        "UUID collision during orchestrator init after {retries} retries"
                    ));
                }
                tracing::warn!("UUID collision during orchestrator init, retrying...");
            }
        }
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Context;
use uuid::Uuid;

use crate::storage::redis::keys::ORCHESTRATORS_SET_KEY;

const SCRIPT: &str = include_str!("dead_orchestrator.lua");

/// Check for stale orchestrator instances.
///
/// Scans the global orchestrators set and returns the IDs of all members whose alive key is no
/// longer present in Redis.
pub(crate) async fn get_dead_orchestrators(client: &redis::Client) -> anyhow::Result<Vec<Uuid>> {
    let script = redis::Script::new(SCRIPT);
    let mut con = client
        .get_multiplexed_async_connection()
        .await
        .context("Failed to get Redis connection for orchestrator check")?;

    let dead_ids = script
        .key(ORCHESTRATORS_SET_KEY)
        .invoke_async::<Vec<String>>(&mut con)
        .await
        .context("Failed to run check_orchestrators script")?;

    dead_ids
        .into_iter()
        .map(|id| {
            Uuid::parse_str(&id)
                .with_context(|| format!("Invalid UUID in orchestrators set: '{id}'"))
        })
        .collect()
}

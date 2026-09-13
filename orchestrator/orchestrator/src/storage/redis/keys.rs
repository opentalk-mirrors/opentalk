// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Redis key definitions for the orchestrator storage backend.
//!
//! ## Key overview
//!
//! | Key pattern                                       | Type              | Value                                      | Description                                                                                     |
//! | ------------------------------------------------- | ----------------- | ------------------------------------------ | ----------------------------------------------------------------------------------------------- |
//! | `ot-orchestrator:service:{service_id}`            | Hash              | `url`, `kind`, `api_key_ids`               | Static data for a registered service instance. `{id}` is the base64-encoded service URL.        |
//! | `ot-orchestrator:service:{service_id}:metrics`    | Hash              | `load` (u8), `accepting_jobs` (bool)       | Current load metrics for a service instance. Updated on every heartbeat.                        |
//! | `ot-orchestrator:service:{service_id}:resources`  | Set               | Resource strings, e.g. `roomserver:{uuid}` | All resources currently managed by this service.                                                |
//! | `ot-orchestrator:services:{kind}`                 | Set               | Service IDs                                | Groups all registered service IDs by kind (roomserver, recorder, transcription).                |
//! | `ot-orchestrator:resource:{resource}:owner`       | String            | Service ID                                 | Reverse lookup: maps a resource to the service ID that owns it.                                 |
//! | `ot-orchestrator:orchestrator:{orch_id}:alive`    | String (expiring) | Orchestrator UUID                          | Liveness indicator. Expires if the orchestrator stops refreshing it. Triggers watchdog cleanup. |
//! | `ot-orchestrator:orchestrator:{orch_id}:services` | Set               | Service IDs                                | All services registered under a specific orchestrator. Used for crash cleanup.                  |
//! | `ot-orchestrator:orchestrators`                   | Set               | Orchestrator UUIDs                         | Global registry of all active orchestrators.                                                    |
use anyhow::Context;
use base64::prelude::{BASE64_STANDARD, Engine};
use opentalk_orchestrator_shared::ServiceKind;
use opentalk_types_common::roomserver::Token;
use redis_args::ToRedisArgs;
use url::Url;

use crate::storage::ServiceResource;

/// The base64 of a service url to avoid special URL character issues in redis keys
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServiceId(String);

impl ServiceId {
    pub fn inner(self) -> String {
        self.0
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ServiceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<&str> for ServiceId {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if let Ok(url) = Url::parse(s) {
            Ok(Self(BASE64_STANDARD.encode(url.as_str())))
        } else {
            let decoded = BASE64_STANDARD
                .decode(s)
                .context("Failed to parse URL or decode BASE64 for service id")?;
            let url_str = String::from_utf8(decoded).context("non-utf8 in service id")?;
            if Url::parse(&url_str).is_ok() {
                Ok(Self(s.to_string()))
            } else {
                Err(anyhow::anyhow!(
                    "Failed to parse URL or decode BASE64 for service id"
                ))
            }
        }
    }
}

impl From<&Url> for ServiceId {
    fn from(url: &Url) -> Self {
        Self(BASE64_STANDARD.encode(url.as_str()))
    }
}

impl TryFrom<&ServiceId> for Url {
    type Error = anyhow::Error;

    fn try_from(value: &ServiceId) -> Result<Self, Self::Error> {
        let decoded = BASE64_STANDARD
            .decode(&value.0)
            .context("Failed to decode BASE64 in service id")?;
        let url_str = String::from_utf8(decoded).context("non-utf8 in service id")?;
        Url::parse(&url_str).context("Failed to parse decoded service id as url")
    }
}

/// Contains static data associated with a specific service instance (url, kind, api_key_ids)
#[derive(Debug, ToRedisArgs)]
#[to_redis_args(fmt = "ot-orchestrator:service:{id}")]
pub struct ServiceInstanceKey {
    pub id: ServiceId,
}

/// Contains the current metrics of a specific service instance (load, accepting_jobs)
///
/// The heartbeat message of each managed service instance contains their current metrics. This key
/// is updated on every heartbeat by the respective managing orchestrator instance.
#[derive(Debug, ToRedisArgs)]
#[to_redis_args(fmt = "ot-orchestrator:service:{id}:metrics")]
pub struct ServiceMetricsKey {
    pub id: ServiceId,
}

/// Contains a list of all [`ServiceResource`]s associated with a specific [`ServiceId`].
#[derive(Debug, ToRedisArgs)]
#[to_redis_args(fmt = "ot-orchestrator:service:{id}:resources")]
pub struct ServiceResourceKey {
    pub id: ServiceId,
}

/// Contains a list of all [`ServiceId`]s related to a specific [`ServiceKind`].
#[derive(Debug, ToRedisArgs)]
#[to_redis_args(fmt = "ot-orchestrator:services:{kind}")]
pub struct ServiceKindKey {
    pub kind: ServiceKind,
}

/// Contains the [`ServiceId`] associated with the given resource
#[derive(Debug, ToRedisArgs)]
#[to_redis_args(fmt = "ot-orchestrator:resource:{resource}:owner")]
pub struct ResourceLookupKey {
    pub resource: ServiceResource,
}

/// Expiring key used as a liveness indicator for a specific orchestrator instance.
#[derive(Debug, ToRedisArgs)]
#[to_redis_args(fmt = "ot-orchestrator:orchestrator:{id}:alive")]
pub struct OrchestratorAliveKey {
    pub id: String,
}

/// Set of service IDs currently managed by a specific orchestrator instance.
/// Used during cleanup to find and remove all services owned by a dead orchestrator.
#[derive(Debug, ToRedisArgs)]
#[to_redis_args(fmt = "ot-orchestrator:orchestrator:{id}:services")]
pub struct OrchestratorServicesKey {
    pub id: String,
}

pub const ORCHESTRATORS_SET_KEY: &str = "ot-orchestrator:orchestrators";

/// Token mapping key for the roomserver signaling proxy
///
/// Maps the token to the associated room id
#[derive(Debug, ToRedisArgs)]
#[to_redis_args(fmt = "ot-orchestrator:roomserver-token:{token}")]
pub struct RoomserverTokenKey {
    pub token: Token,
}

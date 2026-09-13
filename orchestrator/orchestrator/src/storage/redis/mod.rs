// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use anyhow::Context;
use opentalk_orchestrator_shared::{
    Metrics, OrchestratorMetrics, RecorderResource, RegisterType, ServiceKind,
    TranscriptionResource,
    services::{InstanceData, ServiceResource},
};
use opentalk_service_auth::ApiKeyId;
use opentalk_types_common::{rooms::RoomId, roomserver::Token};
use redis::IntoConnectionInfo;
use url::Url;
use uuid::Uuid;

use crate::{
    service_instance::registration::ServiceRegistration,
    storage::{
        AddInstanceError, OrchestratorStorage, ROOMSERVER_TOKEN_EXPIRY,
        redis::{
            keys::{
                ResourceLookupKey, RoomserverTokenKey, ServiceId, ServiceInstanceKey,
                ServiceKindKey, ServiceMetricsKey, ServiceResourceKey,
            },
            peer_monitor::PeerMonitor,
            scripts::AddInstanceScriptError,
        },
    },
    tasks::Tasks,
};

/// The time to live of the orchestrators alive key, an expired key leads to a dead orchestrator
/// cleanup
pub const KEEP_ALIVE_TTL: u64 = 5;
/// The time between each orchestrator alive key refresh
pub const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(2);

pub(crate) mod keys;
mod peer_monitor;
pub(crate) mod scripts;
#[cfg(test)]
mod tests;
mod util;

#[derive(Debug)]
pub(crate) struct RedisStorage {
    pub(crate) client: redis::Client,
    pub(crate) orchestrator_id: Uuid,
    pub(crate) roomserver_token_expiry: Duration,
}

impl RedisStorage {
    /// Initialize the redis storage
    ///
    /// The orchestrator makes use of the RESP3 protocol. A minimum redis version of 7.2 is
    /// required.
    pub(crate) async fn init(tasks: &mut Tasks, redis_url: &str) -> anyhow::Result<Self> {
        let connection_info = redis_url.into_connection_info()?;
        let redis_connection_info = connection_info
            .redis_settings()
            .clone()
            .set_protocol(redis::ProtocolVersion::RESP3);
        let connection_info = connection_info.set_redis_settings(redis_connection_info);

        let client =
            redis::Client::open(connection_info).context("Failed to create Redis client")?;

        let orchestrator_id = scripts::init(&client).await?;

        let peer_monitor = PeerMonitor::init(client.clone(), orchestrator_id).await?;

        tasks.spawn("redis-peer-monitor", |shutdown_rx| {
            peer_monitor.run(shutdown_rx)
        });

        let dead_orchestrators = scripts::get_dead_orchestrators(&client).await?;

        if !dead_orchestrators.is_empty() {
            scripts::cleanup(&client, &dead_orchestrators)
                .await
                .context("Failed to cleanup dead orchestrators in redis")?;
        }

        Ok(Self {
            client,
            orchestrator_id,
            roomserver_token_expiry: ROOMSERVER_TOKEN_EXPIRY,
        })
    }
}

#[async_trait::async_trait]
impl OrchestratorStorage for RedisStorage {
    async fn get_orchestrator_metrics(&self) -> anyhow::Result<OrchestratorMetrics> {
        Ok(OrchestratorMetrics {
            roomservers: scripts::get_service_metrics::<RoomId>(&self.client).await?,
            recorders: scripts::get_service_metrics::<RecorderResource>(&self.client).await?,
            transcription: scripts::get_service_metrics::<TranscriptionResource>(&self.client)
                .await?,
        })
    }

    async fn set_service_metrics(
        &self,
        url: &Url,
        _kind: ServiceKind,
        metrics: Metrics,
    ) -> anyhow::Result<()> {
        let service_id = ServiceId::from(url);

        redis::cmd("HSET")
            .arg(ServiceMetricsKey { id: service_id })
            .arg("load")
            .arg(metrics.load)
            .arg("accepting_jobs")
            .arg(metrics.accepting_jobs)
            .query_async::<()>(&mut self.client.get_multiplexed_async_connection().await?)
            .await
            .context("Failed to set service metrics in Redis")?;

        Ok(())
    }

    async fn add_instance(
        &self,
        ServiceRegistration {
            address,
            register_type,
            api_key_ids,
            metrics,
        }: ServiceRegistration,
    ) -> Result<(), AddInstanceError> {
        let (kind, resources) = match register_type {
            RegisterType::Recorder(register_recorder) => {
                let resources = register_recorder
                    .rooms
                    .into_iter()
                    .map(Into::into)
                    .collect();

                (ServiceKind::Recorder, resources)
            }
            RegisterType::Roomserver(register_roomserver) => {
                let resources = register_roomserver
                    .rooms
                    .into_iter()
                    .map(Into::into)
                    .collect();

                (ServiceKind::Roomserver, resources)
            }
            RegisterType::Transcription(register_transcription) => {
                let resources = register_transcription
                    .rooms
                    .into_iter()
                    .map(Into::into)
                    .collect();

                (ServiceKind::Transcription, resources)
            }
        };

        let result = scripts::add_instance(
            &self.client,
            address,
            kind,
            api_key_ids,
            metrics.load,
            metrics.accepting_jobs,
            resources,
            self.orchestrator_id,
        )
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(AddInstanceScriptError::Registration(registration_error)) => {
                Err(registration_error.into())
            }
            Err(internal) => Err(internal).context("Failed to add instance")?,
        }
    }

    async fn get_instance(
        &self,
        url: &Url,
        kind: ServiceKind,
    ) -> anyhow::Result<Option<InstanceData>> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .context("Failed to get Redis connection")?;

        let id = ServiceId::from(url);

        let (api_key_ids, service_ids, (load, accepting_jobs)): (
            Option<String>,
            Vec<String>,
            (Option<u8>, Option<bool>),
        ) = redis::pipe()
            .atomic()
            .cmd("HGET")
            .arg(ServiceInstanceKey { id: id.clone() })
            .arg("api_key_ids")
            .cmd("SMEMBERS")
            .arg(ServiceKindKey { kind })
            .cmd("HMGET")
            .arg(ServiceMetricsKey { id: id.clone() })
            .arg("load")
            .arg("accepting_jobs")
            .query_async(&mut con)
            .await?;

        let service_ids = service_ids
            .into_iter()
            .map(|sid| ServiceId::try_from(sid.as_str()))
            .collect::<anyhow::Result<Vec<ServiceId>>>()?;

        if service_ids.is_empty() || !service_ids.contains(&id) {
            return Ok(None);
        }

        let (Some(api_key_ids), Some(load), Some(accepting_jobs)) =
            (api_key_ids, load, accepting_jobs)
        else {
            return Ok(None);
        };

        let api_key_ids: Vec<ApiKeyId> = serde_json::from_str::<Vec<String>>(&api_key_ids)
            .context("Failed to deserialize api_key_ids")?
            .into_iter()
            .map(ApiKeyId::from)
            .collect();

        Ok(Some(InstanceData {
            metrics: Metrics {
                load,
                accepting_jobs,
            },
            api_key_ids,
        }))
    }

    async fn get_instances_for_resource(
        &self,
        resource: &ServiceResource,
    ) -> anyhow::Result<Option<(Url, InstanceData)>> {
        scripts::get_instance_by_resource(&self.client, resource).await
    }

    async fn remove_instance(&self, url: &Url, kind: ServiceKind) -> anyhow::Result<()> {
        scripts::remove_instance(&self.client, self.orchestrator_id, url, kind).await
    }

    async fn remove_service_resource(
        &self,
        url: &Url,
        resource: ServiceResource,
    ) -> anyhow::Result<()> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .context("Failed to get Redis connection")?;

        let id = ServiceId::from(url);

        let resource_key = ServiceResourceKey { id };
        let resource_lookup_key = ResourceLookupKey { resource };

        let resource_str = resource.to_string();

        let _: (i64, ()) = redis::pipe()
            .atomic()
            .cmd("SREM")
            .arg(resource_key)
            .arg(resource_str)
            .cmd("DEL")
            .arg(resource_lookup_key)
            .query_async(&mut con)
            .await?;

        Ok(())
    }

    async fn select_instance_for_resource(
        &self,
        resource: ServiceResource,
    ) -> anyhow::Result<(Url, InstanceData)> {
        scripts::select_instance(&self.client, resource).await
    }

    async fn add_roomserver_token(&self, room_id: RoomId, token: Token) -> anyhow::Result<()> {
        redis::cmd("SET")
            .arg(RoomserverTokenKey { token })
            .arg(room_id)
            .arg("EX")
            .arg(self.roomserver_token_expiry.as_secs())
            .query_async::<()>(&mut self.client.get_multiplexed_async_connection().await?)
            .await
            .context("Failed to set roomserver token in Redis")
    }

    async fn consume_roomserver_token(&self, token: &Token) -> anyhow::Result<Option<RoomId>> {
        let (room_id, _) = redis::pipe()
            .atomic()
            .cmd("GET")
            .arg(RoomserverTokenKey { token: *token })
            .cmd("DEL")
            .arg(RoomserverTokenKey { token: *token })
            .query_async::<(Option<RoomId>, ())>(
                &mut self.client.get_multiplexed_async_connection().await?,
            )
            .await
            .context("Failed to consume roomserver token from Redis")?;

        Ok(room_id)
    }

    #[cfg(test)]
    async fn set_roomserver_token_expiry(&mut self, expiry: Duration) {
        self.roomserver_token_expiry = expiry
    }

    #[cfg(test)]
    async fn assert_empty(&self) -> anyhow::Result<()> {
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg("ot-orchestrator:*")
            .query_async(&mut self.client.get_multiplexed_async_connection().await?)
            .await
            .context("Failed to get all keys from Redis")?;

        if keys.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Redis storage is not empty, found keys: {:?}",
                keys
            ))
        }
    }

    #[cfg(test)]
    async fn get_all_resources(&self) -> anyhow::Result<Vec<String>> {
        let services: Vec<String> = redis::cmd("KEYS")
            .arg("ot-orchestrator:resource:*:owner")
            .query_async(&mut self.client.get_multiplexed_async_connection().await?)
            .await
            .context("Failed to get all resource keys from Redis")?;

        let mut resources = Vec::with_capacity(services.len());

        for key in services {
            if let Some(resource_str) = key.strip_prefix("ot-orchestrator:resource:")
                && let Some(resource) = resource_str.strip_suffix(":owner")
            {
                resources.push(resource.into());
            }
        }

        Ok(resources)
    }

    #[cfg(test)]
    async fn get_all_services(&self) -> anyhow::Result<Vec<String>> {
        let mut con = self
            .client
            .get_multiplexed_async_connection()
            .await
            .context("Failed to get Redis connection")?;

        let service_set_keys: Vec<String> = redis::cmd("KEYS")
            .arg("ot-orchestrator:orchestrator:*:services")
            .query_async(&mut con)
            .await
            .context("Failed to get orchestrator service set keys from Redis")?;

        let mut urls = Vec::new();

        for set_key in service_set_keys {
            let service_ids: Vec<String> = redis::cmd("SMEMBERS")
                .arg(&set_key)
                .query_async(&mut con)
                .await
                .context("Failed to get service members from Redis")?;

            for id in service_ids {
                let url: Option<String> = redis::cmd("HGET")
                    .arg(format!("ot-orchestrator:service:{id}"))
                    .arg("url")
                    .query_async(&mut con)
                    .await
                    .context("Failed to get url from service instance key")?;

                if let Some(url) = url {
                    urls.push(url);
                }
            }
        }

        Ok(urls)
    }
}

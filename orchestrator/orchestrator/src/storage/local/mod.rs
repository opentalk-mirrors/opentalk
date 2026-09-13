// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{HashMap, HashSet, hash_map::Entry},
    sync::Arc,
};

use anyhow::{Context, Result};
use opentalk_orchestrator_shared::{
    Metrics, OrchestratorMetrics, RecorderResource, RegisterType, ServiceKind,
    TranscriptionResource,
    error::RegistrationError,
    services::{ResourceType, ServiceState},
};
use opentalk_types_common::{rooms::RoomId, roomserver::Token};
use tokio::sync::{Mutex, RwLock};
use url::Url;

use crate::{
    service_instance::registration::ServiceRegistration,
    storage::{
        AddInstanceError, InstanceData, OrchestratorStorage, ServiceResource,
        local::token_store::{LocalTokenStore, TokenExpiry},
    },
};

pub(crate) mod token_store;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub(crate) struct LocalStorage {
    recorder_services: Arc<RwLock<HashMap<Url, ServiceState<RecorderResource>>>>,
    roomserver_services: Arc<RwLock<HashMap<Url, ServiceState<RoomId>>>>,
    transcription_services: Arc<RwLock<HashMap<Url, ServiceState<TranscriptionResource>>>>,
    token_store: Arc<Mutex<LocalTokenStore>>,
}

impl LocalStorage {
    pub fn new() -> Self {
        Self {
            recorder_services: Default::default(),
            roomserver_services: Default::default(),
            transcription_services: Default::default(),
            token_store: Arc::new(Mutex::new(LocalTokenStore::new())),
        }
    }

    async fn insert_service<T: ResourceType + Clone + Into<ServiceResource>>(
        services: &RwLock<HashMap<Url, ServiceState<T>>>,
        address: Url,
        instance_data: InstanceData,
        managed_resources: HashSet<T>,
    ) -> Result<(), RegistrationError> {
        let mut lock = services.write().await;

        let mut colliding_resources: Vec<ServiceResource> = vec![];

        for resource in &managed_resources {
            for data in lock.values() {
                if data.managed_resources.contains(resource) {
                    colliding_resources.push(resource.clone().into());
                }
            }
        }

        if !colliding_resources.is_empty() {
            return Err(RegistrationError::ResourceConflict(colliding_resources));
        }

        match lock.entry(address) {
            Entry::Occupied(_) => Err(RegistrationError::AddressAlreadyInUse),
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(ServiceState {
                    instance_data,
                    managed_resources,
                });
                Ok(())
            }
        }
    }

    async fn get_service<T: ResourceType>(
        services: &RwLock<HashMap<Url, ServiceState<T>>>,
        resource: T,
    ) -> Option<(Url, InstanceData)> {
        let services = services.read().await;

        for (url, data) in services.iter() {
            if data.managed_resources.contains(&resource) {
                return Some((url.clone(), data.instance_data.clone()));
            }
        }

        None
    }

    async fn select_service<T: ResourceType>(
        services: &RwLock<HashMap<Url, ServiceState<T>>>,
        resource: T,
    ) -> anyhow::Result<(Url, InstanceData)> {
        let mut lock = services.write().await;

        let mut lowest_load_instance: Option<(Url, &mut ServiceState<T>)> = None;

        for (url, data) in lock.iter_mut() {
            if data.managed_resources.contains(&resource) {
                return Ok((url.clone(), data.instance_data.clone()));
            }

            let metrics = &data.instance_data.metrics;
            if !metrics.accepting_jobs {
                continue;
            }

            let lowest_load = lowest_load_instance
                .as_ref()
                .map(|(_, state)| state.instance_data.metrics.load)
                .unwrap_or(100);

            if metrics.load == lowest_load {
                let lowest_load_resource_count = lowest_load_instance
                    .as_ref()
                    .map(|(_, state)| state.managed_resources.len())
                    .unwrap_or(usize::MAX);
                let resource_count = data.managed_resources.len();
                // if the load is equal, select the instance with the lowest number of managed
                // resources
                if resource_count < lowest_load_resource_count {
                    lowest_load_instance = Some((url.clone(), data));
                }
            } else if metrics.load < lowest_load {
                lowest_load_instance = Some((url.clone(), data));
            }
        }

        let (url, service_state) = lowest_load_instance.context("No instance available")?;

        service_state.managed_resources.insert(resource);

        Ok((url, service_state.instance_data.clone()))
    }

    async fn remove_service<T: ResourceType>(
        services: &RwLock<HashMap<Url, ServiceState<T>>>,
        address: &Url,
    ) -> anyhow::Result<()> {
        let mut lock = services.write().await;

        lock.remove(address);

        Ok(())
    }
}

#[async_trait::async_trait]
impl OrchestratorStorage for LocalStorage {
    async fn get_orchestrator_metrics(&self) -> anyhow::Result<OrchestratorMetrics> {
        let recorder_services = self.recorder_services.read().await;
        let roomserver_services = self.roomserver_services.read().await;
        let transcription_services = self.transcription_services.read().await;

        Ok(OrchestratorMetrics {
            roomservers: roomserver_services.clone(),
            recorders: recorder_services.clone(),
            transcription: transcription_services.clone(),
        })
    }

    async fn set_service_metrics(
        &self,
        url: &Url,
        kind: ServiceKind,
        metrics: Metrics,
    ) -> anyhow::Result<()> {
        match kind {
            ServiceKind::Recorder => {
                let mut lock = self.recorder_services.write().await;
                let recorder = lock
                    .get_mut(url)
                    .context("Failed to update recorder metrics for {url}, unknown service")?;
                recorder.set_metrics(metrics);
            }
            ServiceKind::Roomserver => {
                let mut lock = self.roomserver_services.write().await;
                let roomserver = lock
                    .get_mut(url)
                    .context("Failed to update roomserver metrics for {url}, unknown service")?;
                roomserver.set_metrics(metrics);
            }
            ServiceKind::Transcription => {
                let mut lock = self.transcription_services.write().await;
                let transcription = lock
                    .get_mut(url)
                    .context("Failed to update transcription metrics for {url}, unknown service")?;
                transcription.set_metrics(metrics);
            }
        };

        Ok(())
    }

    async fn add_instance(
        &self,
        registration: ServiceRegistration,
    ) -> Result<(), AddInstanceError> {
        let instance_data = InstanceData {
            metrics: registration.metrics,
            api_key_ids: registration.api_key_ids,
        };

        match registration.register_type {
            RegisterType::Recorder(register_recorder) => {
                Self::insert_service(
                    &self.recorder_services,
                    registration.address,
                    instance_data,
                    register_recorder.rooms,
                )
                .await?
            }
            RegisterType::Roomserver(register_roomserver) => {
                Self::insert_service(
                    &self.roomserver_services,
                    registration.address,
                    instance_data,
                    register_roomserver.rooms,
                )
                .await?
            }
            RegisterType::Transcription(register_transcription) => {
                Self::insert_service(
                    &self.transcription_services,
                    registration.address,
                    instance_data,
                    register_transcription.rooms,
                )
                .await?
            }
        }

        Ok(())
    }

    async fn get_instance(
        &self,
        url: &Url,
        kind: ServiceKind,
    ) -> anyhow::Result<Option<InstanceData>> {
        let instance_data = match kind {
            ServiceKind::Recorder => self
                .recorder_services
                .read()
                .await
                .get(url)
                .map(|s| s.instance_data.clone()),
            ServiceKind::Roomserver => self
                .roomserver_services
                .read()
                .await
                .get(url)
                .map(|s| s.instance_data.clone()),
            ServiceKind::Transcription => self
                .transcription_services
                .read()
                .await
                .get(url)
                .map(|s| s.instance_data.clone()),
        };

        Ok(instance_data)
    }

    async fn get_instances_for_resource(
        &self,
        resource: &ServiceResource,
    ) -> anyhow::Result<Option<(Url, InstanceData)>> {
        match resource {
            ServiceResource::Roomserver(room_id) => {
                Ok(Self::get_service(&self.roomserver_services, *room_id).await)
            }
            ServiceResource::Recorder(recorder_resource) => {
                Ok(Self::get_service(&self.recorder_services, *recorder_resource).await)
            }
            ServiceResource::Transcription(transcription_resource) => {
                Ok(Self::get_service(&self.transcription_services, *transcription_resource).await)
            }
        }
    }

    async fn remove_instance(&self, url: &Url, service_kind: ServiceKind) -> anyhow::Result<()> {
        match service_kind {
            ServiceKind::Recorder => Self::remove_service(&self.recorder_services, url).await,
            ServiceKind::Roomserver => Self::remove_service(&self.roomserver_services, url).await,
            ServiceKind::Transcription => {
                Self::remove_service(&self.transcription_services, url).await
            }
        }
    }

    async fn remove_service_resource(
        &self,
        url: &Url,
        resource: ServiceResource,
    ) -> anyhow::Result<()> {
        match resource {
            ServiceResource::Roomserver(room_id) => {
                let mut lock = self.roomserver_services.write().await;
                if let Some(service_state) = lock.get_mut(url) {
                    service_state.managed_resources.remove(&room_id);
                }
            }
            ServiceResource::Recorder(recorder_resource) => {
                let mut lock = self.recorder_services.write().await;
                if let Some(service_state) = lock.get_mut(url) {
                    service_state.managed_resources.remove(&recorder_resource);
                }
            }
            ServiceResource::Transcription(transcription_resource) => {
                let mut lock = self.transcription_services.write().await;
                if let Some(service_state) = lock.get_mut(url) {
                    service_state
                        .managed_resources
                        .remove(&transcription_resource);
                }
            }
        }

        Ok(())
    }

    async fn select_instance_for_resource(
        &self,
        resource: ServiceResource,
    ) -> anyhow::Result<(Url, InstanceData)> {
        match resource {
            ServiceResource::Roomserver(room_id) => {
                Self::select_service(&self.roomserver_services, room_id).await
            }
            ServiceResource::Recorder(recorder_resource) => {
                Self::select_service(&self.recorder_services, recorder_resource).await
            }
            ServiceResource::Transcription(transcription_resource) => {
                Self::select_service(&self.transcription_services, transcription_resource).await
            }
        }
    }

    async fn add_roomserver_token(&self, room_id: RoomId, token: Token) -> anyhow::Result<()> {
        let mut token_store = self.token_store.lock().await;
        token_store.remove_expired_entries();

        if let Some(room) = token_store.tokens.insert(token, room_id) {
            anyhow::bail!("UUID collision, token {token} already exists for room {room}");
        }

        token_store.expiry_set.insert(TokenExpiry::new_now(token));

        Ok(())
    }

    async fn consume_roomserver_token(&self, token: &Token) -> anyhow::Result<Option<RoomId>> {
        let mut token_store = self.token_store.lock().await;
        token_store.remove_expired_entries();
        Ok(token_store.tokens.remove(token))
    }

    #[cfg(test)]
    async fn set_roomserver_token_expiry(&mut self, expiry: std::time::Duration) {
        self.token_store.lock().await.expiry = expiry
    }

    #[cfg(test)]
    async fn assert_empty(&self) -> anyhow::Result<()> {
        let is_empty = self.roomserver_services.read().await.is_empty()
            && self.recorder_services.read().await.is_empty()
            && self.transcription_services.read().await.is_empty();

        if is_empty {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Expected empty storage, got:\nRoomserver: {:?}\nRecorder: {:?}\nTranscription: {:?}",
                self.roomserver_services.read().await,
                self.recorder_services.read().await,
                self.transcription_services.read().await
            ))
        }
    }

    #[cfg(test)]
    async fn get_all_resources(&self) -> anyhow::Result<Vec<String>> {
        let mut resources = Vec::new();

        for state in self.roomserver_services.read().await.values() {
            resources.extend(
                state
                    .managed_resources
                    .iter()
                    .map(|r| ServiceResource::Roomserver(*r).to_string()),
            );
        }
        for state in self.recorder_services.read().await.values() {
            resources.extend(
                state
                    .managed_resources
                    .iter()
                    .map(|r| ServiceResource::Recorder(*r).to_string()),
            );
        }
        for state in self.transcription_services.read().await.values() {
            resources.extend(
                state
                    .managed_resources
                    .iter()
                    .map(|r| ServiceResource::Transcription(*r).to_string()),
            );
        }

        Ok(resources)
    }

    #[cfg(test)]
    async fn get_all_services(&self) -> anyhow::Result<Vec<String>> {
        let mut services = Vec::new();

        services.extend(
            self.roomserver_services
                .read()
                .await
                .keys()
                .map(|u| u.to_string()),
        );
        services.extend(
            self.recorder_services
                .read()
                .await
                .keys()
                .map(|u| u.to_string()),
        );
        services.extend(
            self.transcription_services
                .read()
                .await
                .keys()
                .map(|u| u.to_string()),
        );

        Ok(services)
    }
}

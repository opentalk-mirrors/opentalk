// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use async_trait::async_trait;
use opentalk_orchestrator_client::{
    Metrics, RegisterRoomserver, RegisterType, ServiceResource, client::StateProvider,
};
use opentalk_roomserver_types::signaling::websocket::SignalingSocket;
use sysinfo::{CpuRefreshKind, RefreshKind, System};

use crate::RoomTaskRegistry;

/// Overall time to wait for the deleted rooms to disappear from the registry.
const ROOM_REMOVAL_TIMEOUT: Duration = Duration::from_secs(2);

/// Time to wait between checks whether the deleted rooms are still in the registry.
const ROOM_REMOVAL_POLL_INTERVAL: Duration = Duration::from_millis(25);

pub struct OrchestratorStateProvider<S: SignalingSocket + 'static> {
    registry: RoomTaskRegistry<S>,
    system: System,
}

impl<S: SignalingSocket + 'static> OrchestratorStateProvider<S> {
    pub fn new(registry: RoomTaskRegistry<S>) -> Self {
        let mut system = System::new_with_specifics(
            RefreshKind::nothing().with_cpu(CpuRefreshKind::nothing().with_cpu_usage()),
        );
        system.refresh_cpu_usage();

        Self { registry, system }
    }

    fn calculate_load(&mut self) -> u8 {
        self.system.refresh_cpu_usage();
        let cpu_usage = self.system.global_cpu_usage();

        (cpu_usage as u8).min(100)
    }
}

#[async_trait]
impl<S: SignalingSocket + 'static> StateProvider for OrchestratorStateProvider<S> {
    async fn register_type(&mut self) -> opentalk_orchestrator_client::RegisterType {
        let rooms = self.registry.room_ids().await;

        RegisterType::Roomserver(RegisterRoomserver { rooms })
    }

    async fn metrics(&mut self) -> Metrics {
        Metrics {
            load: self.calculate_load(),
            accepting_jobs: true,
        }
    }

    async fn on_resource_collision(&mut self, resources: &[ServiceResource]) -> anyhow::Result<()> {
        let colliding_rooms = resources
            .iter()
            .filter_map(|resource| match resource {
                ServiceResource::Roomserver(room_id) => Some(*room_id),
                other => {
                    tracing::error!(
                        "Received invalid conflicting resource from orchestrator: {other}"
                    );
                    None
                }
            })
            .collect::<Vec<_>>();

        tracing::warn!("Orchestrator reported resource collision for rooms: {colliding_rooms:?}");

        for room_id in &colliding_rooms {
            tracing::info!("Deleting room '{room_id}' ");
            self.registry.delete_room(*room_id).await;
        }

        // Room deletion only signals the room task to close, its not immediate and can lead to race
        // conditions
        tokio::time::timeout(ROOM_REMOVAL_TIMEOUT, async {
            loop {
                let remaining_rooms = self.registry.room_ids().await;
                if colliding_rooms
                    .iter()
                    .all(|room_id| !remaining_rooms.contains(room_id))
                {
                    return;
                }
                tokio::time::sleep(ROOM_REMOVAL_POLL_INTERVAL).await;
            }
        })
        .await
        .map_err(|_| anyhow::anyhow!("timed out waiting for rooms to be removed from the registry"))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use opentalk_orchestrator_client::RecorderResource;
    use opentalk_roomserver_common::application_state::ApplicationState;
    use opentalk_roomserver_types::room_parameters::RoomParameters;
    use opentalk_types_api_internal::module_assets::Quota;
    use opentalk_types_common::{rooms::RoomId, utils::ExampleData as _};
    use tokio::sync::watch;

    use super::*;
    use crate::{
        ModuleRegistry,
        mocking::socket::MockSocket,
        storage::{
            memory_asset_storage::MemoryAssetStorage,
            memory_module_storage::MemoryModuleResourceStorage,
        },
        task::context::RoomTaskContext,
    };

    fn create_task_context(app_state: watch::Receiver<ApplicationState>) -> RoomTaskContext {
        let asset_storage = Arc::new(MemoryAssetStorage::new(Quota {
            total: None,
            used: 0,
        }));
        let module_resources = Arc::new(MemoryModuleResourceStorage::new());

        RoomTaskContext {
            module_registry: ModuleRegistry::new().into(),
            asset_storage,
            module_resources,
            settings: Arc::default(),
            app_state,
        }
    }

    async fn create_registry_with_room() -> (RoomTaskRegistry<MockSocket>, RoomId) {
        let registry = RoomTaskRegistry::<MockSocket>::new(None);
        let (_app_state_sender, app_state) = watch::channel(ApplicationState::Running);

        let room_id = RoomId::from_u128(0);
        let ctx = create_task_context(app_state);
        registry
            .put_room(ctx, room_id, RoomParameters::example_data().into())
            .await
            .unwrap();

        (registry, room_id)
    }

    #[test_log::test(tokio::test)]
    async fn on_resource_collision_deletes_the_colliding_room() {
        let (registry, room_id) = create_registry_with_room().await;
        let registry_handle = registry.clone();

        let mut provider = OrchestratorStateProvider::new(registry);

        provider
            .on_resource_collision(&[ServiceResource::Roomserver(room_id)])
            .await
            .unwrap();

        assert!(!registry_handle.room_ids().await.contains(&room_id));
    }

    #[test_log::test(tokio::test)]
    async fn on_resource_collision_ignores_non_roomserver_resources() {
        let (registry, room_id) = create_registry_with_room().await;
        let registry_handle = registry.clone();

        let mut provider = OrchestratorStateProvider::new(registry);

        let result = provider
            .on_resource_collision(&[
                ServiceResource::Roomserver(room_id),
                ServiceResource::Recorder(RecorderResource {
                    room_id,
                    breakout_id: None,
                }),
            ])
            .await;

        assert!(result.is_ok());
        assert!(!registry_handle.room_ids().await.contains(&room_id));
    }
}

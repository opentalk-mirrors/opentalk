// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{collections::HashSet, sync::Arc};

use opentalk_orchestrator_client::{RoomserverEvent, client::OrchestratorHandle};
use opentalk_roomserver_types::{
    core::RoomCloseReason, room_action::RoomAction, room_parameters::RoomParameters,
    room_parameters_patch::RoomParametersPatch, signaling::websocket::SignalingSocket,
};
use opentalk_types_common::{rooms::RoomId, users::UserId};
use tokio::sync::{Notify, RwLock};

use crate::{
    RoomTaskApiError,
    room_map::RoomMap,
    task::{
        RoomTask,
        context::RoomTaskContext,
        handle::{RoomTaskHandle, RoomTaskHandleError},
    },
};

/// The room task registry
///
/// Holds a list over all active rooms and their [`RoomTaskHandle`].
#[derive(Default, Debug)]
pub struct RoomTaskRegistry<Socket: SignalingSocket + 'static> {
    rooms: Arc<RwLock<RoomMap<Socket>>>,
    room_removed: Arc<Notify>,
    orchestrator_handle: Option<OrchestratorHandle>,
}

// Manually implementing clone so that we don't require [`Socket`] to be
// Clone as well.
impl<Socket: SignalingSocket> Clone for RoomTaskRegistry<Socket> {
    fn clone(&self) -> Self {
        Self {
            rooms: self.rooms.clone(),
            room_removed: self.room_removed.clone(),
            orchestrator_handle: self.orchestrator_handle.clone(),
        }
    }
}

impl<Socket: SignalingSocket> RoomTaskRegistry<Socket> {
    /// Creates a new [`RoomTaskRegistry`] wi th default values
    pub fn new(orchestrator_handle: Option<OrchestratorHandle>) -> Self {
        Self {
            rooms: Arc::default(),
            room_removed: Arc::default(),
            orchestrator_handle,
        }
    }

    /// Spawns a new room task and adds it to the registry.
    ///
    /// Returns [`Created`] when a new room was created otherwise [`Updated`] is returned.
    ///
    /// [`Created`]: RoomAction::Created
    /// [`Updated`]: RoomAction::Updated
    pub async fn put_room(
        &self,
        ctx: RoomTaskContext,
        room_id: RoomId,
        room_parameters: Arc<RoomParameters>,
    ) -> Result<(RoomAction, RoomTaskHandle<Socket>), RoomTaskHandleError<Socket>> {
        let rooms = self.rooms.write().await;

        if let Some(task_handle) = rooms.map().get(&room_id) {
            task_handle
                .set_parameters((*room_parameters).clone())
                .await?;
            return Ok((RoomAction::Updated, task_handle.clone()));
        }

        let created_by = room_parameters.created_by.id;
        let (task_handle, future_room) = RoomTask::setup(ctx, room_id, room_parameters);

        self.insert(room_id, created_by, rooms, &task_handle, future_room)
            .await;

        Ok((RoomAction::Created, task_handle))
    }

    pub async fn patch_room(
        &self,
        room_id: RoomId,
        patch: RoomParametersPatch,
    ) -> Result<RoomAction, RoomTaskHandleError<Socket>> {
        let rooms = self.rooms.read().await;

        let Some(task_handle) = rooms.map().get(&room_id) else {
            return Err(RoomTaskApiError::NotFound.into());
        };

        task_handle.patch_parameters(patch).await?;
        Ok(RoomAction::Updated)
    }

    pub async fn delete_room(&self, room_id: RoomId) {
        let rooms = self.rooms.read().await;

        // Deleting a non existing room is treated as success.
        // Removing the room from the registry is not necessary here. The room will be removed when
        // the room task completes, see `insert()`.
        if let Some(task_handle) = rooms.map().get(&room_id) {
            task_handle.close(RoomCloseReason::RoomDeleted).await;
        };
    }

    async fn insert(
        &self,
        room_id: RoomId,
        created_by: UserId,
        mut rooms: tokio::sync::RwLockWriteGuard<'_, RoomMap<Socket>>,
        task_handle: &RoomTaskHandle<Socket>,
        future_room: impl Future<Output = ()> + Send + 'static,
    ) {
        let this = self.clone();

        tokio::spawn(async move {
            future_room.await;
            this.remove_room(room_id).await;
        });

        rooms.insert(room_id, created_by, task_handle.clone());
    }

    /// Spawns a new room task if it does not already exists
    pub async fn create_if_not_exists(
        &self,
        ctx: RoomTaskContext,
        room_id: RoomId,
        room_parameters: Arc<RoomParameters>,
    ) {
        let rooms = self.rooms.write().await;

        if rooms.map().get(&room_id).is_some() {
            // Room already exists
            return;
        }

        let created_by = room_parameters.created_by.id;
        let (task_handle, join_handle) = RoomTask::setup(ctx, room_id, room_parameters);

        self.insert(room_id, created_by, rooms, &task_handle, join_handle)
            .await;
    }

    /// Checks if the requested room id exists and refreshes the idle timeout if it does
    pub async fn ensure_room_exists(&self, room_id: &RoomId) -> bool {
        let rooms = self.rooms.read().await;

        let Some(handle) = rooms.map().get(room_id) else {
            return false;
        };

        match handle.refresh_idle_timeout().await {
            Ok(()) => true,
            Err(RoomTaskHandleError::Gone { .. }) => false,
            Err(e) => {
                tracing::error!("Unexpected error while refreshing idle timeout: {e}");
                false
            }
        }
    }

    pub async fn is_guest_access_allowed(&self, room_id: RoomId) -> Option<bool> {
        self.rooms
            .read()
            .await
            .map()
            .get(&room_id)?
            .is_guest_access_allowed()
            .await
    }

    pub async fn allowed_origins(&self, room_id: RoomId) -> Option<Vec<String>> {
        let rooms = self.rooms.read().await;
        rooms.map().get(&room_id)?.allowed_origins().await
    }

    /// Get the [`RoomTaskHandle`] for the specified [`RoomId`]
    pub async fn get_task_handle(&self, room_id: &RoomId) -> Option<RoomTaskHandle<Socket>> {
        self.rooms.read().await.map().get(room_id).cloned()
    }

    /// Get the [`RoomTaskHandle`]s for all rooms created by the specified [`UserId`]
    pub async fn task_handles_by_creator(
        &self,
        creator: UserId,
    ) -> Vec<(RoomId, RoomTaskHandle<Socket>)> {
        self.rooms.read().await.handles_by_creator(creator)
    }

    /// Removes the room from the registry
    ///
    /// This will also destroy the related [`RoomTask`]
    async fn remove_room(&self, room_id: RoomId) {
        tracing::trace!("Remove room task handle from registry: {room_id}");
        let mut rooms = self.rooms.write().await;

        rooms.remove(room_id);
        self.room_removed.notify_waiters();

        if let Some(handle) = &self.orchestrator_handle
            && let Err(e) = handle
                .send_event(RoomserverEvent::RemoveRoom(room_id))
                .await
        {
            tracing::error!("Failed to notify orchestrator about removed room: {e}");
        }
    }

    /// Returns all known room ids
    pub async fn room_ids(&self) -> HashSet<RoomId> {
        self.rooms.read().await.map().keys().copied().collect()
    }

    /// Wait until all pending room task have finished.
    ///
    /// ## Warning
    ///
    /// Use this function with care, as this will acquire a lock for `pending_room_tasks`.
    /// This will block new tasks from being added to the pending tasks.
    pub async fn wait_for_room_closed(&self) {
        // Wait for all room tasks to finish, acquire the lock in every iteration to not cause a
        // deadlock
        while !self.rooms.read().await.map().is_empty() {
            tracing::trace!(
                "Wait for room task removal, remaining tasks: {}",
                self.rooms.read().await.map().len()
            );
            self.room_removed.notified().await;
        }
    }
}

#[cfg(test)]
#[cfg(feature = "mock")]
mod tests {
    use std::{assert_matches, sync::Arc, time::Duration};

    use opentalk_orchestrator_client::client::OrchestratorHandle;
    use opentalk_orchestrator_shared::{Event, RoomserverEvent};
    use opentalk_roomserver_common::application_state::ApplicationState;
    use opentalk_roomserver_types::{
        client_parameters::ClientParameters,
        core::{CoreEvent, RoomCloseReason},
        room_action::RoomAction,
        room_parameters::RoomParameters,
        signaling::websocket::CloseFrame,
    };
    use opentalk_types_api_internal::module_assets::Quota;
    use opentalk_types_common::{rooms::RoomId, utils::ExampleData as _};
    use tokio::sync::{mpsc, watch};

    use crate::{
        ModuleRegistry, RoomTaskRegistry,
        mocking::{participant::create_participant_connection, socket::MockSocket},
        storage::{
            memory_asset_storage::MemoryAssetStorage,
            memory_module_storage::MemoryModuleResourceStorage,
        },
        task::context::RoomTaskContext,
    };

    #[test_log::test(tokio::test)]
    async fn room_is_removed_after_idle_timeout() {
        let registry = RoomTaskRegistry::<MockSocket>::new(None);
        let (_app_state_sender, app_state) = watch::channel(ApplicationState::Running);

        let room_id = RoomId::from_u128(1);
        // build room parameter without any modules
        let mut parameter = RoomParameters::example_data();
        parameter.room_idle_timeout = Duration::from_secs(0);
        parameter.module_settings.retain(|_, _| false);

        let ctx = create_task_context(app_state);
        let (action, ..) = registry
            .put_room(ctx, room_id, parameter.into())
            .await
            .unwrap();

        assert_eq!(RoomAction::Created, action);

        tokio::time::timeout(Duration::from_millis(500), registry.wait_for_room_closed())
            .await
            .unwrap();

        let handle = registry.get_task_handle(&room_id).await;
        assert!(handle.is_none());
    }

    #[test_log::test(tokio::test)]
    async fn room_is_removed_after_shutdown() {
        // use a high RoomTask idle timeout to prevent stopping because of the timeout.
        let registry = RoomTaskRegistry::<MockSocket>::new(None);
        let (app_state_sender, app_state) = watch::channel(ApplicationState::Running);

        let room_id = RoomId::from_u128(1);
        // build room parameter without any modules
        let mut parameter = RoomParameters::example_data();
        parameter.room_idle_timeout = Duration::from_secs(99999);
        parameter.module_settings.retain(|_, _| false);

        let ctx = create_task_context(app_state);
        let (action, ..) = registry
            .put_room(ctx, room_id, parameter.into())
            .await
            .unwrap();

        assert_eq!(RoomAction::Created, action);
        app_state_sender
            .send(ApplicationState::ShuttingDown)
            .unwrap();

        tokio::time::timeout(Duration::from_millis(500), registry.wait_for_room_closed())
            .await
            .unwrap();

        let handle = registry.get_task_handle(&room_id).await;
        assert!(handle.is_none());
    }

    #[test_log::test(tokio::test)]
    async fn delete_room_notifies_participants_and_orchestrator() {
        let (orchestrator_tx, mut orchestrator_rx) = mpsc::channel(1);
        let orchestrator_handle = OrchestratorHandle(orchestrator_tx);
        let registry = RoomTaskRegistry::<MockSocket>::new(Some(orchestrator_handle));
        let (_app_state_sender, app_state) = watch::channel(ApplicationState::Running);

        let room_id = RoomId::from_u128(1);
        let ctx = create_task_context(app_state);
        let (action, ..) = registry
            .put_room(ctx, room_id, RoomParameters::example_data().into())
            .await
            .unwrap();
        assert_eq!(RoomAction::Created, action);

        let handle = registry.get_task_handle(&room_id).await.unwrap();
        let (socket, participant) = create_participant_connection();
        handle
            .accept_signaling_socket(socket, ClientParameters::example_data())
            .await
            .unwrap();
        let mut participant = participant
            .enter_lobby()
            .await
            .unwrap()
            .enter_room(None)
            .await
            .unwrap();

        // Delete the room via the registry, as the web api does.
        registry.delete_room(room_id).await;

        // The participant is notified that the room is closing.
        let event = participant.receive::<CoreEvent>().await.unwrap();
        assert_matches!(
            event.payload,
            CoreEvent::Closing {
                reason: RoomCloseReason::RoomDeleted
            }
        );

        // The participant's connection is then closed with a close frame.
        let close_frame = participant.receive_close_frame().await.unwrap();
        assert_eq!(
            close_frame,
            Some(CloseFrame {
                code: 1000,
                reason: "closed by server".to_owned(),
            })
        );

        // The orchestrator is notified that the room was removed.
        let event = orchestrator_rx.recv().await.unwrap();
        assert_eq!(
            event,
            Event::Roomserver(RoomserverEvent::RemoveRoom(room_id))
        );
    }

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
}

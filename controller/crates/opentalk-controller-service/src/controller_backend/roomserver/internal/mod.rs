// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::Duration,
};

use actix_ws::{Message, ProtocolError};
use http::{
    HeaderMap,
    header::{AUTHORIZATION, Entry},
};
use opentalk_asset_storage::{ObjectStorage, StorageNotifier};
use opentalk_controller_settings::{Settings, SettingsProvider};
use opentalk_controller_utils::deletion::{StopRoomBackend, StopRoomError};
use opentalk_inventory::{Inventory, InventoryProvider};
use opentalk_roomserver_room::{
    ApplicationState, ModuleRegistry, RoomTaskApiError, RoomTaskContext, RoomTaskHandleError,
    RoomTaskRegistry, SignalingClientContext, TokenStore, settings::Task,
};
use opentalk_roomserver_types::{
    api::RoomServerAccess,
    client_parameters::ClientParameters,
    livekit_proxy::{LiveKitProxyRequest, PreparedSocket, websocket::LiveKitSocket},
    room_parameters_patch::RoomParametersPatch,
    signaling::websocket::SignalingSocketMessage,
};
use opentalk_roomserver_web_api::livekit_proxy::LiveKitProxyBackend;
use opentalk_types_api_v1::{error::ApiError, rooms::RoomResource};
use opentalk_types_common::{
    features::FeatureId, modules::ModuleId, rooms::RoomId, roomserver::Token, users::UserId,
};
use snafu::ResultExt;
use tokio::sync::{Mutex, broadcast, mpsc, watch, watch::Sender};
use url::Url;

use crate::{
    Whatever,
    controller_backend::roomserver::{
        RoomServerBackend, SignalingHandler, SignalingProxyBackend, build_room_parameters,
        internal::{asset_storage::AssetStorage, module_resources::ModuleResources},
        websocket_adapter::WebSocketAdapter,
    },
};

mod asset_storage;
mod module_resources;

const LIVEKIT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const LIVEKIT_TIMEOUT: Duration = Duration::from_secs(10);

/// A roomserver backend that is running embedded in the controller.
pub(crate) struct InternalRoomServer {
    room_tasks: RoomTaskRegistry<WebSocketAdapter>,
    module_registry: Arc<ModuleRegistry>,
    app_state: Sender<ApplicationState>,
    /// A list of eligible participants and their join tokens
    token_store: Arc<Mutex<TokenStore<SignalingClientContext>>>,
    settings: Arc<Task>,
    livekit_client: reqwest::Client,

    settings_provider: SettingsProvider,
    inventory_provider: Arc<dyn InventoryProvider>,
    storage: Arc<ObjectStorage>,
    storage_notifier: Arc<dyn StorageNotifier>,
}

impl InternalRoomServer {
    /// Create a new internal roomserver backend
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        room_tasks: RoomTaskRegistry<WebSocketAdapter>,
        settings_provider: SettingsProvider,
        inventory_provider: Arc<dyn InventoryProvider>,
        storage: Arc<ObjectStorage>,
        storage_notifier: Arc<dyn StorageNotifier>,
        room_task_settings: Task,
        module_registry: ModuleRegistry,
        shutdown: broadcast::Receiver<()>,
    ) -> Result<Self, Whatever> {
        let app_state = Self::spawn_shutdown_task(shutdown);
        let livekit_client = reqwest::ClientBuilder::new()
            .connect_timeout(LIVEKIT_CONNECT_TIMEOUT)
            .timeout(LIVEKIT_TIMEOUT)
            .build()
            .whatever_context("Failed to build livekit client")?;

        Ok(Self {
            room_tasks,
            module_registry: Arc::new(module_registry),
            app_state,
            token_store: Arc::new(Mutex::new(TokenStore::new())),
            settings: Arc::new(room_task_settings),
            livekit_client,
            settings_provider,
            inventory_provider,
            storage,
            storage_notifier,
        })
    }

    fn room_task_context(&self, room_owner: UserId) -> RoomTaskContext {
        let asset_storage = AssetStorage::new(
            self.settings_provider.clone(),
            Arc::clone(&self.inventory_provider),
            Arc::clone(&self.storage),
            Arc::clone(&self.storage_notifier),
            room_owner,
        );
        let module_resources = ModuleResources::new(Arc::clone(&self.inventory_provider));

        RoomTaskContext {
            module_registry: Arc::clone(&self.module_registry),
            asset_storage: Arc::new(asset_storage),
            module_resources: Arc::new(module_resources),
            settings: Arc::clone(&self.settings),
            app_state: self.app_state.subscribe(),
        }
    }

    fn spawn_shutdown_task(mut shutdown: broadcast::Receiver<()>) -> Sender<ApplicationState> {
        let (shutdown_sender, _) = watch::channel(ApplicationState::Running);

        let sender = shutdown_sender.clone();
        _ = tokio::spawn(async move {
            _ = shutdown.recv().await;
            if let Err(err) = sender.send(ApplicationState::ShuttingDown) {
                log::debug!("Failed to send shutdown signal: {err}");
            }
        });

        shutdown_sender
    }
}

impl SignalingProxyBackend for InternalRoomServer {}

#[async_trait::async_trait]
impl RoomServerBackend for InternalRoomServer {
    #[tracing::instrument(level = "debug", skip(self, inventory, settings, room), fields(%room_id = room.id))]
    async fn request_access(
        &self,
        inventory: &mut dyn Inventory,
        settings: Arc<Settings>,
        module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
        room: RoomResource,
        client_parameters: ClientParameters,
        host: Url,
    ) -> Result<RoomServerAccess, ApiError> {
        let room_id = room.id;
        let task_handle = self.room_tasks.get_task_handle(&room_id).await;

        if let Some(task_handle) = task_handle {
            // Room already exists
            // Refresh the idle timeout of the room
            if let Err(err) = task_handle.refresh_idle_timeout().await {
                log::error!("Failed to refresh idle timeout of room {room_id}: {err}");
                return Err(ApiError::internal().with_message("Failed to refresh idle timeout"));
            }

            // Ensure the user isn't banned
            if let Some(user_id) = client_parameters.kind.user_id() {
                task_handle.reject_if_banned(user_id).await?;
            }

            if client_parameters.kind.is_guest_or_callin() {
                match task_handle.is_guest_access_allowed().await {
                    Some(true) => {}
                    Some(false) => {
                        log::debug!(
                            "Guest access is disabled for room {room_id}, rejecting access"
                        );
                        // Do not leak the existence of the room to guests if guest access is disabled
                        return Err(ApiError::not_found());
                    }
                    None => {
                        log::error!(
                            "Failed to check guest access for room {room_id}, rejecting access"
                        );
                        // Do not leak the existence of the room
                        return Err(ApiError::not_found());
                    }
                }
            }
        } else {
            if client_parameters.kind.is_guest_or_callin() && room.guest_access.is_disabled() {
                // Do not leak the existence of the room to guests if guest access is disabled
                return Err(ApiError::not_found());
            }

            // Room needs to be created
            let room_parameters =
                build_room_parameters(inventory, settings, room, module_features).await?;
            let ctx = self.room_task_context(room_parameters.created_by.id);
            self.room_tasks
                .create_if_not_exists(ctx, room_id, room_parameters.into())
                .await;
        }

        let token = self
            .token_store
            .lock()
            .await
            .create_token(SignalingClientContext::new(room_id, client_parameters));

        Ok(RoomServerAccess {
            public_url: host,
            token,
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn patch_room_parameters(
        &self,
        room_id: RoomId,
        patch: RoomParametersPatch,
    ) -> Result<(), ApiError> {
        match self.room_tasks.patch_room(room_id, patch).await {
            Ok(_) | Err(RoomTaskHandleError::ApiError(RoomTaskApiError::NotFound)) => Ok(()),
            Err(err) => {
                log::debug!("Failed to patch room {room_id}: {err}");
                Err(err.into())
            }
        }
    }
}

#[async_trait::async_trait]
impl StopRoomBackend for InternalRoomServer {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn stop_room(&self, room_id: RoomId) -> Result<(), StopRoomError> {
        self.room_tasks.delete_room(room_id).await;
        Ok(())
    }
}

#[async_trait::async_trait]
impl LiveKitProxyBackend for InternalRoomServer {
    async fn connect_upstream_socket(
        &self,
        ws_request: LiveKitProxyRequest,
    ) -> Result<PreparedSocket, ApiError> {
        let Some(task_handle) = self.room_tasks.get_task_handle(&ws_request.room_id).await else {
            return Err(ApiError::not_found());
        };

        task_handle
            .prepare_proxy_socket(ws_request)
            .await
            .map_err(Into::into)
    }

    async fn connect_downstream_socket(
        &self,
        ws_request: LiveKitProxyRequest,
        upstream_socket: PreparedSocket,
        socket: Box<dyn LiveKitSocket>,
    ) -> Result<(), ApiError> {
        let Some(task_handle) = self.room_tasks.get_task_handle(&ws_request.room_id).await else {
            return Err(ApiError::not_found());
        };

        task_handle
            .accept_livekit_socket(ws_request, upstream_socket, socket)
            .await?;

        Ok(())
    }

    async fn proxy_livekit_validate(
        &self,
        room_id: RoomId,
        mut headers: HeaderMap,
        raw_query: Option<String>,
        v1: bool,
    ) -> Result<reqwest::Response, ApiError> {
        let Some(task_handle) = self.room_tasks.get_task_handle(&room_id).await else {
            return Err(ApiError::not_found());
        };

        let mut livekit_service_url = task_handle.livekit_service_url().await?;
        {
            let mut segments = livekit_service_url.path_segments_mut().map_err(|()| {
                log::error!("Invalid livekit URL, cannot be base");
                ApiError::internal()
            })?;
            let _ = segments.push("rtc");
            if v1 {
                let _ = segments.push("v1");
            }
            let _ = segments.push("validate");
        }
        livekit_service_url.set_query(raw_query.as_deref());

        let auth_headers = match headers.entry(AUTHORIZATION) {
            Entry::Occupied(occupied) => {
                let (_, values) = occupied.remove_entry_mult();

                values
                    .into_iter()
                    .map(|value| (AUTHORIZATION, value))
                    .collect()
            }
            Entry::Vacant(_) => HeaderMap::new(),
        };

        self.livekit_client
            .post(livekit_service_url)
            .headers(auth_headers)
            .send()
            .await
            .map_err(|err| {
                log::error!("Failed to send validate request to livekit: {err}");
                ApiError::internal()
            })
    }
}

#[async_trait::async_trait]
impl SignalingHandler for InternalRoomServer {
    async fn consume_signaling_token(
        &self,
        token: &Token,
    ) -> Result<SignalingClientContext, ApiError> {
        self.token_store
            .lock()
            .await
            .consume_token(token)
            .ok_or_else(|| {
                log::debug!("invalid or expired token");
                ApiError::not_found()
            })
    }

    async fn accept_signaling_connection(
        &self,
        ctx: SignalingClientContext,
        incoming: mpsc::Receiver<Result<Message, ProtocolError>>,
        outgoing: mpsc::Sender<SignalingSocketMessage>,
    ) -> Result<(), ApiError> {
        let SignalingClientContext {
            room_id,
            client_parameters,
        } = ctx;

        let task_handle = self
            .room_tasks
            .get_task_handle(&room_id)
            .await
            .ok_or_else(|| {
                log::debug!("room not found");
                ApiError::not_found().with_message("Room not found")
            })?;

        let adapter = WebSocketAdapter::new(incoming, outgoing);

        task_handle
            .accept_signaling_socket(adapter, client_parameters)
            .await
            .map_err(|err| {
                log::error!("Failed to attach signaling socket to room {room_id}: {err}");
                ApiError::internal().with_message("Failed to attach signaling socket to room task")
            })?;

        Ok(())
    }
}

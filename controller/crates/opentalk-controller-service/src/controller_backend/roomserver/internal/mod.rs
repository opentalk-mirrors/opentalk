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
    client_allowed::{ClientAllowed, ClientNotAllowedReason},
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
        AuthorizedClientContext, RoomServerBackend, SignalingHandler, SignalingProxyBackend,
        build_room_parameters,
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
    async fn request_access<'a>(
        &self,
        inventory: &mut (dyn Inventory + 'a),
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

            match task_handle
                .is_client_allowed(&client_parameters.kind)
                .await?
            {
                ClientAllowed::Yes => {}
                ClientAllowed::No { reason } => {
                    let err = match reason {
                        ClientNotAllowedReason::Banned => ApiError::banned(),
                        ClientNotAllowedReason::GuestAccessDisabled => {
                            log::debug!(
                                "Guest access is disabled for room {room_id}, rejecting access"
                            );
                            // Do not leak the existence of the room to guests if guest access is
                            // disabled
                            ApiError::not_found()
                        }
                        ClientNotAllowedReason::E2eeEnabled => {
                            log::debug!(
                                "Client is not allowed when e2ee is enabled, rejecting access"
                            );
                            ApiError::e2ee_enabled()
                        }
                    };

                    return Err(err);
                }
            };
        } else {
            // Check services before guest access, so services receive the correct `e2ee_enabled`
            // reason instead of the `not_found` that is sent to guests.
            if client_parameters.kind.is_service_or_callin() && room.e2e_encryption {
                log::debug!(
                    "Service attempted to request a token for a room with e2ee enabled, rejecting access"
                );
                return Err(ApiError::e2ee_enabled());
            }

            if client_parameters.kind.is_guest_or_callin() && room.guest_access.is_disabled() {
                log::debug!(
                    "Guest attempted to request a token for a room with guest access disabled, rejecting access"
                );
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

    async fn authorize_client(
        &self,
        ctx: SignalingClientContext,
    ) -> Result<AuthorizedClientContext, ApiError> {
        // Unlike [`RoomServerBackend::request_access`], this does not hide the existence of the
        // room from guests: reaching the signaling endpoint requires a valid token, which is itself
        // proof that the room exists and that access was granted when the token was issued.
        match self
            .room_tasks
            .is_client_allowed(&ctx.client_parameters.kind, ctx.room_id)
            .await?
        {
            ClientAllowed::Yes => Ok(AuthorizedClientContext(ctx)),
            ClientAllowed::No { reason } => Err(match reason {
                ClientNotAllowedReason::Banned => ApiError::banned(),
                ClientNotAllowedReason::GuestAccessDisabled => ApiError::guest_access_disabled(),
                ClientNotAllowedReason::E2eeEnabled => ApiError::e2ee_enabled(),
            }),
        }
    }

    async fn accept_signaling_connection(
        &self,
        ctx: AuthorizedClientContext,
        incoming: mpsc::Receiver<Result<Message, ProtocolError>>,
        outgoing: mpsc::Sender<SignalingSocketMessage>,
    ) -> Result<(), ApiError> {
        let SignalingClientContext {
            room_id,
            client_parameters,
        } = ctx.into_inner();

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

#[cfg(test)]
mod test {
    use http::StatusCode;
    use opentalk_asset_storage::NoOpStorageNotifier;
    use opentalk_controller_settings::test_util::settings_provider_from_example_raw_settings;
    use opentalk_inventory::{MockInventory, MockInventoryProvider};
    use opentalk_roomserver_types::{
        client_parameters::ClientKind, public_user_profile::PublicUserProfile, room_kind::RoomKind,
        room_parameters::RoomParameters,
    };
    use opentalk_types_common::{roomserver::DeviceSecret, utils::ExampleData};
    use opentalk_types_signaling::Role;
    use pretty_assertions::assert_eq;

    use super::*;

    const ROOM_ID: RoomId = RoomId::nil();

    /// Builds an [`InternalRoomServer`] for testing.
    ///
    /// The returned [`broadcast::Sender`] must be kept alive for the duration of the test.
    /// Dropping it closes the shutdown channel, which moves the application state to
    /// [`ApplicationState::ShuttingDown`] and terminates all running room tasks.
    fn test_roomserver() -> (InternalRoomServer, broadcast::Sender<()>) {
        let (shutdown, shutdown_receiver) = broadcast::channel(1);

        let roomserver = InternalRoomServer::new(
            RoomTaskRegistry::new(None),
            settings_provider_from_example_raw_settings(),
            Arc::new(MockInventoryProvider::new()),
            // Requesting access never touches the object storage, so a broken placeholder is
            // sufficient here.
            Arc::new(ObjectStorage::broken()),
            Arc::new(NoOpStorageNotifier),
            Task::default(),
            ModuleRegistry::new(),
            shutdown_receiver,
        )
        .expect("Failed to build the internal roomserver");

        (roomserver, shutdown)
    }

    fn recorder_client_parameters() -> ClientParameters {
        ClientParameters {
            device_secret: DeviceSecret::example_data(),
            kind: ClientKind::Recorder {
                room: RoomKind::Main,
            },
            role: Role::User,
        }
    }

    fn call_in_client_parameters() -> ClientParameters {
        ClientParameters {
            device_secret: DeviceSecret::example_data(),
            kind: ClientKind::CallIn,
            role: Role::User,
        }
    }

    fn registered_client_parameters() -> ClientParameters {
        ClientParameters {
            device_secret: DeviceSecret::example_data(),
            kind: ClientKind::Registered {
                profile: PublicUserProfile::example_data(),
            },
            role: Role::User,
        }
    }

    fn guest_client_parameters() -> ClientParameters {
        ClientParameters {
            device_secret: DeviceSecret::example_data(),
            kind: ClientKind::Unregistered,
            role: Role::User,
        }
    }

    fn room_resource(e2e_encryption: bool) -> RoomResource {
        RoomResource {
            id: ROOM_ID,
            e2e_encryption,
            ..RoomResource::example_data()
        }
    }

    fn host() -> Url {
        Url::parse("http://localhost:11311").expect("Host URL must be valid")
    }

    /// Spawns a running room task for [`ROOM_ID`] in the roomserver's registry.
    async fn spawn_room(roomserver: &InternalRoomServer, e2e_encryption: bool) {
        let room_parameters = RoomParameters {
            e2e_encryption,
            ..RoomParameters::example_data()
        };
        let ctx = roomserver.room_task_context(room_parameters.created_by.id);

        roomserver
            .room_tasks
            .create_if_not_exists(ctx, ROOM_ID, room_parameters.into())
            .await;
    }

    #[tokio::test]
    async fn request_access_recorder_e2ee_enabled_room_not_spawned() {
        let (roomserver, _shutdown) = test_roomserver();

        let error = roomserver
            .request_access(
                &mut MockInventory::new(),
                settings_provider_from_example_raw_settings().get(),
                BTreeMap::new(),
                room_resource(true),
                recorder_client_parameters(),
                host(),
            )
            .await
            .expect_err("Recorders must not get access to an e2ee room");

        assert_eq!(error.status, StatusCode::FORBIDDEN);
        assert_eq!(error.body.code, "e2ee_enabled");

        // The room must not have been created as a side effect of the rejected request
        assert!(
            roomserver
                .room_tasks
                .get_task_handle(&ROOM_ID)
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn request_access_recorder_e2ee_enabled_room_spawned() {
        let (roomserver, _shutdown) = test_roomserver();
        spawn_room(&roomserver, true).await;

        // The room resource deliberately reports `e2e_encryption: false` to assert that the
        // running room task is the source of truth once the room exists.
        let error = roomserver
            .request_access(
                &mut MockInventory::new(),
                settings_provider_from_example_raw_settings().get(),
                BTreeMap::new(),
                room_resource(false),
                recorder_client_parameters(),
                host(),
            )
            .await
            .expect_err("Recorders must not get access to an e2ee room");

        assert_eq!(error.status, StatusCode::FORBIDDEN);
        assert_eq!(error.body.code, "e2ee_enabled");
    }

    #[tokio::test]
    async fn request_access_call_in_e2ee_enabled_room_not_spawned() {
        let (roomserver, _shutdown) = test_roomserver();

        let error = roomserver
            .request_access(
                &mut MockInventory::new(),
                settings_provider_from_example_raw_settings().get(),
                BTreeMap::new(),
                room_resource(true),
                call_in_client_parameters(),
                host(),
            )
            .await
            .expect_err("Call-in must not get access to an e2ee room");

        assert_eq!(error.status, StatusCode::FORBIDDEN);
        assert_eq!(error.body.code, "e2ee_enabled");

        // The room must not have been created as a side effect of the rejected request
        assert!(
            roomserver
                .room_tasks
                .get_task_handle(&ROOM_ID)
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn request_access_call_in_e2ee_enabled_room_spawned() {
        let (roomserver, _shutdown) = test_roomserver();
        spawn_room(&roomserver, true).await;

        // The room resource deliberately reports `e2e_encryption: false` to assert that the
        // running room task is the source of truth once the room exists.
        let error = roomserver
            .request_access(
                &mut MockInventory::new(),
                settings_provider_from_example_raw_settings().get(),
                BTreeMap::new(),
                room_resource(false),
                call_in_client_parameters(),
                host(),
            )
            .await
            .expect_err("Call-in must not get access to an e2ee room");

        assert_eq!(error.status, StatusCode::FORBIDDEN);
        assert_eq!(error.body.code, "e2ee_enabled");
    }

    #[tokio::test]
    async fn authorize_client_rejects_recorder_in_e2ee_room() {
        let (roomserver, _shutdown) = test_roomserver();
        spawn_room(&roomserver, true).await;

        let error = roomserver
            .authorize_client(SignalingClientContext::new(
                ROOM_ID,
                recorder_client_parameters(),
            ))
            .await
            .expect_err("Recorders must not open a signaling connection to an e2ee room");

        assert_eq!(error.status, StatusCode::FORBIDDEN);
        assert_eq!(error.body.code, "e2ee_enabled");
    }

    #[tokio::test]
    async fn authorize_client_rejects_call_in_in_e2ee_room() {
        let (roomserver, _shutdown) = test_roomserver();
        spawn_room(&roomserver, true).await;

        let error = roomserver
            .authorize_client(SignalingClientContext::new(
                ROOM_ID,
                call_in_client_parameters(),
            ))
            .await
            .expect_err("Call-in must not open a signaling connection to an e2ee room");

        assert_eq!(error.status, StatusCode::FORBIDDEN);
        assert_eq!(error.body.code, "e2ee_enabled");
    }

    /// Guards against the e2ee check rejecting more client kinds than intended, which would make
    /// e2ee rooms unjoinable for everyone.
    #[tokio::test]
    async fn authorize_client_allows_registered_user_in_e2ee_room() {
        let (roomserver, _shutdown) = test_roomserver();
        spawn_room(&roomserver, true).await;

        let _authorized = roomserver
            .authorize_client(SignalingClientContext::new(
                ROOM_ID,
                registered_client_parameters(),
            ))
            .await
            .expect("Registered users must be able to join an e2ee room");
    }

    /// Unlike [`RoomServerBackend::request_access`], the signaling endpoint reports a disabled
    /// guest access as `403` instead of hiding the room behind a `404`.
    #[tokio::test]
    async fn authorize_client_rejects_guest_when_guest_access_is_disabled() {
        let (roomserver, _shutdown) = test_roomserver();
        // `RoomParameters::example_data` disables guest access
        spawn_room(&roomserver, false).await;

        let error = roomserver
            .authorize_client(SignalingClientContext::new(
                ROOM_ID,
                guest_client_parameters(),
            ))
            .await
            .expect_err("Guests must not join a room with guest access disabled");

        assert_eq!(error.status, StatusCode::FORBIDDEN);
        assert_eq!(error.body.code, "guest_access_disabled");
    }
}

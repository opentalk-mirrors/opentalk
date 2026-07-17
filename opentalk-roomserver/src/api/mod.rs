// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{net::SocketAddr, sync::Arc};

use anyhow::{Context as _, Result};
use async_trait::async_trait;
use axum::{
    extract::{MatchedPath, OriginalUri},
    http::{Request, Response},
    response::IntoResponse,
    serve::Listener as _,
};
use futures::{StreamExt, stream};
use opentalk_roomserver_common::{
    settings::{ControllerConfig, Settings},
    token_store::TokenStore,
};
use opentalk_roomserver_modules::setup_registry;
use opentalk_roomserver_room::{
    ModuleRegistry, RoomTaskRegistry,
    storage::{
        controller_asset_storage::ControllerAssetStorage,
        controller_module_storage::ControllerModuleStorage,
        memory_asset_storage::MemoryAssetStorage,
        memory_module_storage::MemoryModuleResourceStorage,
    },
    task::context::RoomTaskContext,
};
use opentalk_roomserver_signaling::storage::{
    assets::provider::AssetStorageProvider, module_resources::provider::ModuleResourceProvider,
};
use opentalk_roomserver_types::{
    api::RoomServerAccess,
    client_parameters::ClientParameters,
    livekit_proxy::{LiveKitProxyRequest, PreparedSocket, websocket::LiveKitSocket},
    room_action::RoomAction,
    room_parameters::{self, RoomParameters},
    room_parameters_patch::RoomParametersPatch,
    signaling::signaling_context::SignalingClientContext,
    tariff_details::TariffDetails,
};
use opentalk_roomserver_web_api::{
    livekit_proxy::{self, LiveKitProxyBackend},
    v1::{self, Backend, RoomBackend, SecurityAddon, user::UserBackend},
};
use opentalk_types_api_internal::{error::ApiError, module_assets::Quota};
use opentalk_types_common::{rooms::RoomId, tariffs::QuotaType, users::UserId};
use reqwest::StatusCode;
use service_probe::{ServiceState, set_service_state};
use tokio::sync::{Mutex, watch};
use tower_http::trace::TraceLayer;
use tracing::{Span, info_span};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    ApplicationState, api::websocket::WebSocketAdapter, tcp_multi_listener::MultiListener,
    wait_shutdown,
};

pub(crate) type Router = axum::Router<Context>;

pub mod signaling;
pub mod websocket;

#[derive(OpenApi)]
#[openapi(
        info(
            title = "OpenTalk RoomServer API",
            description = "Specifies the endpoints and structure of the OpenTalk RoomServer Web API",
        ),
        tags(
            (name = "v1::rooms", description = "Endpoints related to rooms"),
            (name = "v1::user", description = "Endpoints related to a user"),
            (name = "v1::signaling", description = "Endpoints related to signaling connections"),
            (name = "livekit_proxy", description = "Endpoints related to the LiveKit proxy"),
        ),
        paths(
            v1::rooms::put_room,
            v1::rooms::patch_room,
            v1::rooms::delete_room,
            v1::rooms::request_token,
            v1::user::post_storage_quota,
            v1::signaling::open_signaling_socket,
            livekit_proxy::proxy_socket,
            livekit_proxy::proxy_validate,
        ),
        components(
            schemas(
                opentalk_types_api_internal::users::PublicUserProfile,
                opentalk_types_common::call_in::CallInId,
                opentalk_types_common::call_in::CallInInfo,
                opentalk_types_common::call_in::CallInPassword,
                opentalk_types_common::call_in::NumericId,
                opentalk_types_common::features::FeatureId,
                opentalk_types_common::rooms::RoomId,
                opentalk_types_common::shared_folders::SharedFolder,
                opentalk_types_common::shared_folders::SharedFolderAccess,
                opentalk_types_common::streaming::StreamingLink,
                opentalk_types_common::tariffs::TariffId,
                opentalk_types_common::users::DisplayName,
                opentalk_types_common::users::UserId,
                opentalk_types_common::users::UserTitle,
                room_parameters::EventContext,
                room_parameters::RoomParameters,
            )
        ),
        modifiers(&SecurityAddon),
    )]
pub(crate) struct ApiDoc;

/// Starts the web server
pub(crate) async fn run_web_server<L>(
    settings: Arc<Settings>,
    addresses: Vec<SocketAddr>,
    room_registry: RoomTaskRegistry<WebSocketAdapter>,
    app_state: watch::Sender<ApplicationState>,
    metric_layer: Option<L>,
) -> anyhow::Result<()>
where
    L: tower::Layer<axum::routing::Route> + Clone + Send + Sync + 'static,
    L::Service: tower::Service<axum::extract::Request> + Clone + Send + Sync + 'static,
    <L::Service as tower::Service<axum::extract::Request>>::Response:
        axum::response::IntoResponse + 'static,
    <L::Service as tower::Service<axum::extract::Request>>::Error:
        Into<std::convert::Infallible> + 'static,
    <L::Service as tower::Service<axum::extract::Request>>::Future: Send + 'static,
{
    let app_state_subscriber = app_state.subscribe();

    let module_registry = setup_registry();

    let ctx = Context {
        settings: Arc::clone(&settings),
        room_tasks: room_registry.clone(),
        token_store: Arc::new(Mutex::new(TokenStore::new())),
        module_registry: Arc::new(module_registry),
        livekit_proxy_client: reqwest::Client::new(),
        app_state,
    };

    let auth_middleware = settings
        .http
        .api_keys
        .auth_middleware()
        .context("Invalid API key configuration")?;

    let mut router = Router::new()
        .merge(livekit_proxy::routes())
        .nest("/v1", v1::routes(ctx.clone(), auth_middleware))
        .with_state(ctx)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);

                    info_span!(
                        "http_request",
                        http.request.method = ?request.method(),
                        http.route = matched_path,
                        http.status_code = tracing::field::Empty,
                    )
                })
                .on_response(|response: &Response<_>, _duration, span: &Span| {
                    span.record(
                        "http.status_code",
                        tracing::field::display(response.status()),
                    );
                    if response.status().is_client_error() {
                        tracing::debug!(
                            "Request failed with status code: {} - (client error)",
                            response.status()
                        );
                    } else if response.status().is_server_error() {
                        tracing::error!(
                            "Request failed with status code: {} - (server error)",
                            response.status()
                        );
                    }
                })
                .on_failure(|fail, _duration, _span: &Span| {
                    tracing::warn!("request failed: {fail:?}");
                }),
        )
        .fallback(not_found_handler);

    if let Some(layer) = metric_layer {
        router = router.layer(layer);
    }

    if settings.http.enable_openapi {
        // TODO: Having this enabled causes the utoipa schema to be cloned and dropped for each
        // request which increases cost by about ~40%
        let mut openapi = ApiDoc::openapi();
        openapi.servers = Some(vec![utoipa::openapi::Server::new("/v1")]);
        router = router.merge(SwaggerUi::new("/swagger").url("/docs/openapi.json", openapi));
    }

    let listener = MultiListener::bind(addresses).await?;

    tracing::info!("Listening on {}", listener.local_addr()?);

    set_service_state(ServiceState::Ready);
    axum::serve(listener, router)
        .with_graceful_shutdown(wait_shutdown(app_state_subscriber))
        .await?;

    // wait for room tasks to close
    room_registry.wait_for_room_closed().await;

    Ok(())
}

async fn not_found_handler(OriginalUri(uri): OriginalUri) -> impl IntoResponse {
    tracing::debug!("Received request for unknown route: {}", uri);

    (StatusCode::NOT_FOUND, "requested path was not found")
}

/// Context for the API endpoints
#[derive(Clone)]
pub(crate) struct Context {
    settings: Arc<Settings>,
    /// Global list of room tasks and their handles
    room_tasks: RoomTaskRegistry<WebSocketAdapter>,
    // A list of eligible participants and their join tokens
    token_store: Arc<Mutex<TokenStore<SignalingClientContext>>>,
    module_registry: Arc<ModuleRegistry>,
    livekit_proxy_client: reqwest::Client,

    app_state: watch::Sender<ApplicationState>,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("settings", &self.settings)
            .field("room_tasks", &self.room_tasks)
            .finish_non_exhaustive()
    }
}

impl Backend for Context {}

#[async_trait]
impl RoomBackend for Context {
    async fn put_room(
        &self,
        room_id: RoomId,
        room_parameters: RoomParameters,
    ) -> Result<RoomAction, opentalk_types_api_internal::error::ApiError> {
        let ctx = self.create_task_context(&room_parameters.tariff);
        let (action, task_handle) = self
            .room_tasks
            .put_room(ctx, room_id, room_parameters.into())
            .await
            .map_err(|err| {
                tracing::info!("Failed to put room {room_id}: {err}");
                err
            })?;

        if !action.is_created() {
            // Refresh the idle timeout if the room was not created with this request
            task_handle.refresh_idle_timeout().await.map_err(|err| {
                tracing::info!("Failed to refresh idle timeout for room {room_id}: {err}");
                err
            })?;
        }

        Ok(action)
    }

    async fn patch_room(
        &self,
        room_id: RoomId,
        patch: RoomParametersPatch,
    ) -> Result<RoomAction, opentalk_types_api_internal::error::ApiError> {
        let action = self
            .room_tasks
            .patch_room(room_id, patch)
            .await
            .inspect_err(|err| tracing::debug!("Failed to patch room {room_id}: {err}"))?;

        Ok(action)
    }

    async fn request_room_token(
        &mut self,
        room_id: RoomId,
        client_parameters: ClientParameters,
        room_parameters: Option<RoomParameters>,
    ) -> Result<RoomServerAccess, ApiError> {
        let task_handle = self.room_tasks.get_task_handle(&room_id).await;

        match (task_handle, room_parameters) {
            // Room doesn't exist and no parameters were provided
            (None, None) => {
                return Err(ApiError::unprocessable_entity()
                    .with_code("room_parameters_missing")
                    .with_message("Room parameters missing"));
            }

            // Room needs to be created
            (None, Some(parameters)) => {
                if client_parameters.kind.is_guest_or_callin() && !parameters.guest_access {
                    return Err(ApiError::forbidden()
                        .with_code("guest_access_disabled")
                        .with_message("Guest access disabled"));
                }

                let ctx = self.create_task_context(&parameters.tariff);
                self.room_tasks
                    .create_if_not_exists(ctx, room_id, parameters.into())
                    .await;
            }

            // Room already exists
            (Some(task_handle), _) => {
                // refresh the idle timeout of the room
                if let Err(e) = task_handle.refresh_idle_timeout().await {
                    tracing::error!("Failed to refresh idle timeout of room {room_id}: {e}");
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
                            return Err(ApiError::forbidden()
                                .with_code("guest_access_disabled")
                                .with_message("Guest access disabled"));
                        }
                        None => {
                            return Err(ApiError::internal()
                                .with_message("Failed to check room guest access"));
                        }
                    }
                }
            }
        }

        let token = self
            .token_store
            .lock()
            .await
            .create_token(SignalingClientContext::new(room_id, client_parameters));

        let public_url = self.settings.http.public_url.clone();

        Ok(RoomServerAccess { public_url, token })
    }

    async fn delete_room(&self, room_id: RoomId) {
        self.room_tasks.delete_room(room_id).await;
    }
}

impl Context {
    fn create_task_context(&self, tariff: &TariffDetails) -> RoomTaskContext {
        let quota = Quota {
            total: tariff.quota(&QuotaType::MaxStorage),
            used: tariff.used_quota(&QuotaType::MaxStorage),
        };

        let (asset_storage, module_resources): (
            Arc<dyn AssetStorageProvider>,
            Arc<dyn ModuleResourceProvider>,
        ) = match &self.settings.controller {
            Some(ControllerConfig { url, api_key }) => {
                let asset_storage =
                    ControllerAssetStorage::new(url.clone(), api_key.clone(), quota);
                let module_resources = ControllerModuleStorage::new(url.clone(), api_key.clone());

                (Arc::new(asset_storage), Arc::new(module_resources))
            }
            None => (
                Arc::new(MemoryAssetStorage::new(quota)),
                Arc::new(MemoryModuleResourceStorage::new()),
            ),
        };

        RoomTaskContext {
            module_registry: Arc::clone(&self.module_registry),
            asset_storage,
            module_resources,
            settings: Arc::clone(&self.settings.task),
            app_state: self.app_state.subscribe(),
        }
    }
}

#[async_trait]
impl LiveKitProxyBackend for Context {
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
        headers: axum::http::HeaderMap,
        raw_query: Option<String>,
    ) -> Result<reqwest::Response, ApiError> {
        let Some(task_handle) = self.room_tasks.get_task_handle(&room_id).await else {
            return Err(ApiError::not_found());
        };

        let mut livekit_service_url = task_handle.livekit_service_url().await?;
        livekit_service_url
            .path_segments_mut()
            .map_err(|()| {
                tracing::error!("Invalid livekit URL, cannot be base");
                ApiError::internal()
            })?
            .push("rtc")
            .push("validate");
        livekit_service_url.set_query(raw_query.as_deref());

        self.livekit_proxy_client
            .post(livekit_service_url)
            .headers(headers)
            .send()
            .await
            .map_err(|err| {
                tracing::error!("Failed to send validate request to livekit: {err}");
                ApiError::internal()
            })
    }
}

#[async_trait]
impl UserBackend for Context {
    async fn post_storage_quota(&self, user_id: UserId, quota: Quota) -> Result<(), ApiError> {
        let handles = self.room_tasks.task_handles_by_creator(user_id).await;
        if handles.is_empty() {
            return Err(
                ApiError::not_found().with_message("No rooms created by the specified user exist")
            );
        }

        let parallel_requests = self
            .settings
            .internal
            .parallel_storage_quota_requests
            .into();

        let results = stream::iter(handles)
            .map(|(room_id, handle)| {
                let quota = quota.clone();
                async move {
                    handle.set_storage_quota(quota).await.inspect_err(|err| {
                        tracing::warn!("Failed to set storage quota for room {room_id}: {err}");
                    })
                }
            })
            .buffer_unordered(parallel_requests)
            .collect::<Vec<_>>()
            .await;

        if let Some(err) = results.into_iter().find_map(Result::err) {
            return Err(err.into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{assert_matches, borrow::Cow, sync::Arc, time::Duration};

    use axum::http::StatusCode;
    use icu_locid::langid;
    use opentalk_roomserver_common::settings::{ControllerConfig, Http};
    use opentalk_roomserver_types::{
        client_parameters::{ClientKind, Role},
        module_settings::ModuleSettings,
        public_user_profile::PublicUserProfile,
        room_parameters::WaitingRoom,
        tariff_details::TariffDetails,
    };
    use opentalk_service_auth::{ApiKey, service::ApiKeys};
    use opentalk_types_api_internal::error::ErrorBody;
    use opentalk_types_common::{
        roomserver::DeviceSecret,
        time::TimeZone,
        users::{DisplayName, UserId, UserInfo, UserTitle},
        utils::ExampleData,
    };
    use pretty_assertions::assert_eq;
    use url::Url;

    use super::*;

    fn test_context() -> Context {
        let (app_state, _) = watch::channel(ApplicationState::Running);

        Context {
            settings: Arc::new(test_settings()),
            room_tasks: RoomTaskRegistry::new(None),
            token_store: Arc::new(Mutex::new(TokenStore::new())),
            module_registry: Arc::new(ModuleRegistry::new()),
            livekit_proxy_client: reqwest::Client::new(),
            app_state,
        }
    }

    /// Creates settings for testing
    pub fn test_settings() -> Settings {
        let port = 11333;
        let address = "localhost".into();
        let public_url = Url::parse(&format!("http://{address}:{port}")).unwrap();
        let controller = ControllerConfig {
            url: Url::parse("http://localhost:8000").unwrap(),
            api_key: ApiKey::new("controller", "secret"),
        };

        Settings {
            http: Http {
                address,
                port,
                api_keys: ApiKeys::new(vec![ApiKey::new("roomserver", "secret")]),
                enable_openapi: true,
                service_url: None,
                public_url,
            },
            controller: Some(controller),
            orchestrator: None,
            monitoring: None,
            metrics: None,
            tracing: None,
            internal: Default::default(),
            task: Arc::default(),
        }
    }

    fn guest_client_parameters() -> ClientParameters {
        ClientParameters {
            device_secret: DeviceSecret::example_data(),
            kind: ClientKind::Guest {
                display_name: DisplayName::from_str_lossy("tester"),
            },
            role: Role::User,
        }
    }

    fn registered_client_parameters() -> ClientParameters {
        ClientParameters {
            device_secret: DeviceSecret::example_data(),
            kind: ClientKind::Registered {
                profile: PublicUserProfile {
                    id: UserId::nil(),
                    email: "example@opentalk.eu".into(),
                    user_info: UserInfo {
                        title: UserTitle::new(),
                        firstname: "Test".into(),
                        lastname: "Tester".into(),
                        display_name: DisplayName::from_str_lossy("tester"),
                        avatar_url: "example.com".into(),
                    },
                    timezone: TimeZone::example_data(),
                },
            },
            role: Role::Moderator,
        }
    }

    fn room_parameters() -> RoomParameters {
        RoomParameters {
            created_by: PublicUserProfile::example_data(),
            password: None,
            guest_access: true,
            waiting_room: WaitingRoom::Disabled,
            call_in: None,
            event: None,
            invite_code: None,
            tariff: TariffDetails::example_data(),
            streaming_targets: vec![],
            show_meeting_details: true,
            e2e_encryption: false,
            module_settings: ModuleSettings::example_data(),
            preferred_language: langid!("en"),
            fallback_language: langid!("en"),
            ws_rate_limit: None,
            allowed_origins: vec!["*".to_string()],
            room_idle_timeout: Duration::from_secs(10),
        }
    }

    #[test_log::test(tokio::test)]
    async fn put_room() {
        let ctx = test_context();

        let id = RoomId::from_u128(0xf4bc4806_a35c_4ce0_bcb3_fb990b287d4c);
        let action = ctx.put_room(id, room_parameters()).await.unwrap();
        assert_eq!(action, RoomAction::Created);

        // TODO add second put_room request and check for UPDATED response once implemented
    }

    #[test_log::test(tokio::test)]
    async fn delete_room() {
        let ctx = test_context();

        let id = RoomId::from_u128(0x1);
        let action = ctx.put_room(id, room_parameters()).await.unwrap();
        assert_eq!(action, RoomAction::Created);

        ctx.delete_room(id).await;
        assert!(ctx.room_tasks.get_task_handle(&id).await.is_none());
    }

    #[test_log::test(tokio::test)]
    async fn request_token() {
        let mut ctx = test_context();

        let token = ctx
            .request_room_token(RoomId::nil(), registered_client_parameters(), None)
            .await;

        assert_matches!(
            token,
            Err(ApiError {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                body: ErrorBody {
                    code: Cow::Borrowed("room_parameters_missing"),
                    ..
                },
                ..
            })
        );

        let token = ctx
            .request_room_token(
                RoomId::nil(),
                registered_client_parameters(),
                Some(room_parameters()),
            )
            .await;

        assert!(token.is_ok());

        let token = ctx
            .request_room_token(RoomId::nil(), guest_client_parameters(), None)
            .await;

        assert!(token.is_ok())
    }

    #[test_log::test(tokio::test)]
    async fn request_token_guests_disabled() {
        let mut ctx = test_context();

        let mut room_parameters = room_parameters();
        room_parameters.guest_access = false;
        let error = ctx
            .request_room_token(
                RoomId::nil(),
                guest_client_parameters(),
                Some(room_parameters.clone()),
            )
            .await
            .unwrap_err();
        assert_eq!(error.status, StatusCode::FORBIDDEN);

        let room_action = ctx.put_room(RoomId::nil(), room_parameters.clone()).await;
        assert_eq!(room_action, Ok(RoomAction::Created));

        // Requesting tokens also fails if the room already exists
        let error = ctx
            .request_room_token(
                RoomId::nil(),
                guest_client_parameters(),
                Some(room_parameters.clone()),
            )
            .await
            .unwrap_err();
        assert_eq!(error.status, StatusCode::FORBIDDEN);
    }
}

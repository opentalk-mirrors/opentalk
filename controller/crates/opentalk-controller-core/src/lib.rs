// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Extensible core library of the *OpenTalk Controller*

use std::{
    fs::File,
    io::BufReader,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, ToSocketAddrs as _},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use actix_cors::Cors;
use actix_web::{App, HttpServer, Scope, web, web::Data};
use exchange_task::{ExchangeHandle, ExchangeTask};
use lapin_pool::RabbitMqPool;
use opentalk_asset_storage::{ObjectStorage, StorageNotifier};
use opentalk_controller_api_actix_web::{v1, well_known};
use opentalk_controller_api_authorization::{
    authorization::Authorizer, middleware::AuthorizationTransform,
};
use opentalk_controller_api_authorization_database::OpenTalkAuthorizerBackend;
use opentalk_controller_service::{
    ControllerBackend, RedisConnection, Whatever,
    controller_backend::roomserver::{self, SignalingProxyBackend},
    oidc::{Cache, OidcTokenHandler, build_oidc_token_handler},
    services::MailService,
};
use opentalk_controller_service_facade::OpenTalkControllerService;
use opentalk_controller_settings::{
    HttpTls, Monitoring, Settings, SettingsProvider, UserSearchBackend, UserSearchBackendKeycloak,
    common::{HttpCorsAllowedOrigin, HttpCorsAllowedOrigins},
};
use opentalk_controller_utils::deletion::StopRoomBackend;
use opentalk_database::Db;
use opentalk_inventory::InventoryProvider;
use opentalk_inventory_database::DatabaseConnectionPool;
use opentalk_jobs::job_runner::JobRunner;
use opentalk_keycloak_admin::{AuthorizedClient, KeycloakAdminClient};
use opentalk_service_auth::service::ApiKeyAuthorization;
use opentalk_types_api_v1::{auth::OidcProvider, error::ApiError};
use rustls_pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use service_probe::{ServiceState, set_service_state, start_probe};
use snafu::{Report, ResultExt, Snafu};
use swagger::WithSwagger as _;
use tokio::{
    signal::{
        ctrl_c,
        unix::{SignalKind, signal},
    },
    sync::broadcast,
    task::JoinError,
    time::sleep,
};
use tracing_actix_web::TracingLogger;

use crate::{
    api::{
        livekit,
        v1::{middleware::metrics::RequestMetrics, response::error::json_error_handler},
    },
    trace::ReducedSpanBuilder,
};

mod exchange_task;
mod metrics;
mod swagger;
mod trace;

pub mod api;

#[derive(Debug, Snafu)]
/// Blocking thread has panicked
pub struct BlockingError {
    source: JoinError,
}

impl From<BlockingError> for ApiError {
    fn from(e: BlockingError) -> Self {
        tracing::error!(
            "REST API threw internal error from blocking error: {}",
            Report::from_error(e)
        );
        Self::internal()
    }
}

type Result<T, E = Whatever> = std::result::Result<T, E>;

/// Custom version of `actix_web::web::block` which retains the current tracing span
pub async fn block<F, R>(f: F) -> Result<R, BlockingError>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let span = tracing::Span::current();

    let fut = actix_rt::task::spawn_blocking(move || span.in_scope(f));

    fut.await.context(BlockingSnafu)
}

/// Controller struct representation containing all fields required to extend and drive the controller
pub struct Controller {
    pub service: Arc<dyn OpenTalkControllerService>,

    /// The roomserver backend used for signaling
    signaling_handler: Option<Arc<dyn SignalingProxyBackend>>,

    /// Settings loaded on [Controller::create]
    pub startup_settings: Arc<Settings>,

    /// Cloneable shared settings, can be used to reload settings from, when receiving the `reload` signal.
    pub settings_provider: SettingsProvider,

    /// Path of the configuration file
    optional_config_path: Option<PathBuf>,

    inventory_provider: Arc<dyn InventoryProvider>,

    oidc_cache: Arc<Cache>,

    storage: Arc<ObjectStorage>,

    storage_notifier: Arc<dyn StorageNotifier>,

    oidc: Arc<dyn OidcTokenHandler>,

    user_search_client: Arc<Option<KeycloakAdminClient>>,

    authorizer: Authorizer,

    stop_room_backend: Arc<dyn StopRoomBackend>,

    /// RabbitMQ connection pool, can be used to create connections and channels
    pub rabbitmq_pool: Arc<Option<Arc<RabbitMqPool>>>,

    /// Handle to the internal message exchange
    pub exchange_handle: ExchangeHandle,

    /// Reload signal which can be triggered by a user.
    /// When received a module should try to re-read it's config and act accordingly.
    ///
    /// `controller.reload.subscribe()` to receive a receiver to the reload-signal.
    pub reload: broadcast::Sender<()>,

    /// Shutdown signal which is triggered when the controller is exiting, either because a fatal error occurred
    /// or a user requested the shutdown.
    ///
    /// `controller.shutdown.subscribe()` to receive a receiver to the reload-signal.
    /// The controller will wait up 10 seconds before forcefully shutting down.
    /// It is tracking the shutdown progress by counting the shutdown-receiver count.
    pub shutdown: broadcast::Sender<()>,

    /// All metrics of the Application
    pub metrics: metrics::CombinedMetrics,
}

impl Controller {
    /// Creates a controller instance based on the optional config path command-line argument.
    pub async fn create(optional_config_path: Option<PathBuf>) -> Result<Self> {
        let settings_provider = load_settings_provider(optional_config_path.as_deref())?;

        tracing::info!("Starting OpenTalk Controller");

        tracing::info!(
            "Global timezone is {}",
            settings_provider.get().defaults.timezone
        );

        let controller = Self::init(settings_provider, optional_config_path)
            .await
            .whatever_context("Failed to init controller")?;

        Ok(controller)
    }

    #[tracing::instrument(err(level = "debug"), skip(settings_provider))]
    async fn init(
        settings_provider: SettingsProvider,
        optional_config_path: Option<PathBuf>,
    ) -> Result<Self> {
        let settings = settings_provider.get();
        let metrics = metrics::CombinedMetrics::try_init(&settings.roomserver.kind)
            .whatever_context("Failed to initialize metrics")?;

        opentalk_db_storage::migrations::migrate_from_url(&settings.database.url)
            .await
            .whatever_context("Failed to migrate database")?;

        let rabbitmq_pool = Arc::new(settings.rabbit_mq.as_ref().map(|config| {
            RabbitMqPool::from_config(
                &config.url,
                config.min_connections,
                config.max_channels_per_connection,
            )
        }));

        // Only use rabbitmq in the exchange when a both rabbitmq and redis are configured.
        // This assumes that the existence of redis means multiple controllers are used
        // and share their signaling state via redis. Only in this case rabbitmq is required
        // in the exchange.
        let exchange_handle = match (settings.redis.is_some(), rabbitmq_pool.as_ref()) {
            (true, Some(rabbitmq_pool)) => ExchangeTask::spawn_with_rabbitmq(rabbitmq_pool.clone())
                .await
                .whatever_context("Failed to spawn exchange task")?,
            _ => ExchangeTask::spawn()
                .await
                .whatever_context("Failed to spawn exchange task")?,
        };

        // Connect to postgres
        let mut db =
            Db::connect(&settings.database).whatever_context("Failed to connect to database")?;
        db.set_metrics(metrics.database.clone());
        let db = Arc::new(db);

        // Connect to MinIO
        let storage = Arc::new(
            ObjectStorage::new(&settings.minio)
                .await
                .whatever_context("Failed to initialize object storage")?,
        );

        let oidc_frontend = &settings.oidc.frontend;
        let oidc_controller = &settings.oidc.controller;

        // Discover OIDC Provider
        let oidc = build_oidc_token_handler(
            oidc_frontend.authority.clone(),
            oidc_controller.authority.clone(),
            oidc_controller.client_id.clone(),
            oidc_controller.client_secret.clone(),
        )
        .await
        .whatever_context("Failed to initialize OIDC Context")?;

        let user_search_client =
            if let Some(UserSearchBackend::Keycloak(UserSearchBackendKeycloak {
                api_base_url,
                client_id,
                client_secret,
                external_id_user_attribute_name: _,
            })) = &settings.user_search_backend
            {
                let authorized_client = AuthorizedClient::new(
                    oidc_controller.authority.clone(),
                    client_id.clone().into(),
                    client_secret.secret().clone(),
                )
                .whatever_context("Failed to initialize authorized client")?;

                Arc::new(Some(
                    KeycloakAdminClient::new(api_base_url.clone(), authorized_client)
                        .whatever_context("Failed to initialize keycloak")?,
                ))
            } else {
                Arc::new(None)
            };

        // Build redis client. Does not check if redis is reachable.
        let redis = settings
            .redis
            .as_ref()
            .map(|r| redis::Client::open(r.url.clone()))
            .transpose()
            .whatever_context("Invalid redis url")?;
        let redis_conn = match redis {
            Some(c) => Some(
                redis::aio::ConnectionManager::new(c)
                    .await
                    .whatever_context("Failed to create redis connection manager")?,
            ),
            None => None,
        };
        let redis_conn =
            redis_conn.map(|c| RedisConnection::new(c).with_metrics(metrics.redis.clone()));
        let oidc_cache = Arc::new(Cache::create(redis_conn.clone()));

        let (shutdown, _) = broadcast::channel::<()>(1);
        let (reload, _) = broadcast::channel::<()>(4);
        let inventory_provider: Arc<dyn InventoryProvider> =
            Arc::new(DatabaseConnectionPool::new(db));

        let mail_service = Arc::new(match rabbitmq_pool.as_ref() {
            Some(rabbitmq_pool) => Some(MailService::new(
                metrics.endpoint.clone(),
                rabbitmq_pool.clone(),
                rabbitmq_pool
                    .create_channel()
                    .await
                    .whatever_context("Failed to create rabbitmq channel")?,
            )),
            None => None,
        });

        let registry = opentalk_roomserver_modules::setup_registry();
        let module_features = registry.module_features();
        let roomserver = roomserver::build(
            &settings.roomserver.kind,
            settings_provider.clone(),
            Arc::clone(&inventory_provider),
            Arc::clone(&storage),
            registry,
            shutdown.subscribe(),
        )?;

        let authorizer_backend = OpenTalkAuthorizerBackend::new(
            inventory_provider.clone(),
            settings_provider.clone(),
            module_features.clone(),
        );
        let authorizer = Authorizer::new(authorizer_backend);

        let backend = {
            let oidc_provider = OidcProvider {
                name: oidc_frontend.client_id.to_string(),
                url: oidc_frontend.authority.to_string(),
            };
            ControllerBackend::new(
                settings_provider.clone(),
                authorizer.clone(),
                inventory_provider.clone(),
                oidc_cache.clone(),
                oidc.clone(),
                oidc_provider,
                storage.clone(),
                mail_service.clone(),
                user_search_client.clone(),
                module_features,
                Arc::clone(&roomserver.backend),
            )
        };

        let service = Arc::new(backend);
        let controller = Self {
            service,
            signaling_handler: roomserver.signaling_handler,
            startup_settings: settings,
            settings_provider,
            storage_notifier: roomserver.storage_notifier,
            optional_config_path,
            inventory_provider,
            oidc_cache,
            storage,
            oidc,
            user_search_client,
            authorizer,
            stop_room_backend: roomserver.backend,
            rabbitmq_pool,
            exchange_handle,
            shutdown,
            reload,
            metrics,
        };

        Ok(controller)
    }

    /// Runs the controller until a fatal error occurred or a shutdown is requested (e.g. SIGTERM).
    pub async fn run(self) -> Result<()> {
        if let Some(Monitoring { port, addr }) = self.startup_settings.monitoring {
            start_probe(addr, port, ServiceState::Up)
                .await
                .whatever_context("Failed to start monitoring")?;
        }

        // Start JobExecutor
        JobRunner::start(
            self.inventory_provider.clone(),
            self.authorizer.clone(),
            Arc::clone(&self.stop_room_backend),
            self.shutdown.subscribe(),
            self.startup_settings.clone(),
        )
        .await
        .whatever_context("Failed to start Job Runner")?;

        // Start HTTP Server
        let http_server = {
            let settings_provider = self.settings_provider.clone();
            let storage = Arc::downgrade(&self.storage);
            let storage_notifier = Arc::downgrade(&self.storage_notifier);
            let inventory_provider = Arc::downgrade(&self.inventory_provider);
            let http_client = Data::new(reqwest::Client::new());

            let oidc_ctx = Arc::downgrade(&self.oidc);
            let shutdown = self.shutdown.clone();

            let user_search_client = Data::from(self.user_search_client);

            let metrics = Data::new(self.metrics);

            let caches = Data::from(self.oidc_cache.clone());
            let service = Data::from(self.service);
            let signaling = Data::new(self.signaling_handler);

            let authorization = AuthorizationTransform::new(self.authorizer.clone());
            let service_auth_middleware = settings_provider
                .get()
                .http
                .service_api_keys
                .clone()
                .map(|keys| keys.auth_middleware())
                .transpose()
                .whatever_context("Failed to build service auth middleware")?;

            HttpServer::new(move || {
                let cors = setup_cors(settings_provider.clone());

                // Unwraps cannot panic. Server gets stopped before dropping the Arc.
                let inventory_provider = Data::from(inventory_provider.upgrade().unwrap());
                let storage = Data::from(storage.upgrade().unwrap());
                let storage_notifier = Data::from(storage_notifier.upgrade().unwrap());

                let oidc_ctx = Data::from(oidc_ctx.upgrade().unwrap());
                let authorizer = Data::new(self.authorizer.clone());

                let authorization = authorization.clone();
                let service_auth_middleware = service_auth_middleware.clone();

                let swagger_service_enabled = !settings_provider.get().endpoints.disable_openapi;

                App::new()
                    .wrap(RequestMetrics::new(metrics.endpoint.clone()))
                    .wrap(cors)
                    .wrap(TracingLogger::<ReducedSpanBuilder>::new())
                    .wrap(api::v1::middleware::headers::Headers {})
                    .app_data(service.clone())
                    .app_data(signaling.clone())
                    .app_data(caches.clone())
                    .app_data(web::JsonConfig::default().error_handler(json_error_handler))
                    .app_data(Data::new(settings_provider.clone()))
                    .app_data(inventory_provider.clone())
                    .app_data(storage)
                    .app_data(oidc_ctx.clone())
                    .app_data(user_search_client.clone())
                    .app_data(authorizer.clone())
                    .app_data(Data::new(shutdown.clone()))
                    .app_data(metrics.clone())
                    .app_data(http_client.clone())
                    .app_data(storage_notifier)
                    .service(well_known::opentalk::api::get)
                    .service(metrics::metrics)
                    .with_swagger_service_if(swagger_service_enabled)
                    .service(internal_service_scope(service_auth_middleware))
                    .service(livekit_scope())
                    .service(v1_scope(
                        settings_provider.clone(),
                        authorizer,
                        inventory_provider.clone(),
                        oidc_ctx.clone(),
                        authorization,
                    ))
            })
        };

        let socket_address = determine_socket_address(
            self.startup_settings.http.addr.as_deref(),
            self.startup_settings.http.port,
        )
        .whatever_context("Unable to determine bind address")?;

        let http_server = if let Some(tls) = &self.startup_settings.http.tls {
            let config = setup_rustls(tls).whatever_context("Failed to setup TLS context")?;

            http_server.bind_rustls_0_23(&socket_address[..], config)
        } else {
            http_server.bind(&socket_address[..])
        };

        let http_server = http_server.with_whatever_context(|_| {
            format!("Failed to bind http server to {socket_address:?}")
        })?;

        set_service_state(ServiceState::Ready);
        tracing::info!("Startup finished");

        let http_server = http_server.disable_signals().run();
        let http_server_handle = http_server.handle();

        let mut reload_signal = signal(SignalKind::hangup())
            .whatever_context("Failed to register SIGHUP signal handler")?;

        actix_rt::spawn(http_server);

        // Wait for either SIGTERM or SIGHUP and handle them accordingly
        loop {
            tokio::select! {
                _ = ctrl_c() => {
                    tracing::info!("Got termination signal, exiting");
                    break;
                }
                _ = reload_signal.recv() => {
                    tracing::info!("Got reload signal, reloading");

                    if let Err(e) = self.settings_provider.reload_from_path_or_standard_paths(self.optional_config_path.as_deref()) {
                        tracing::error!("Failed to reload settings, {}", Report::from_error(e));
                        continue
                    }

                    // discard result, might fail if no one is subscribed
                    let _ = self.reload.send(());
                }
            }
        }

        // ==== Begin shutdown sequence ====

        // Send shutdown signals to all tasks within our application
        let _ = self.shutdown.send(());

        // then stop HTTP server
        http_server_handle.stop(true).await;

        // Check in a 1 second interval for 10 seconds if all tasks have exited
        // by inspecting the receiver count of the broadcast-channel
        for _ in 0..10 {
            let receiver_count = self.shutdown.receiver_count();

            if receiver_count > 0 {
                tracing::debug!("Waiting for {receiver_count} tasks to be stopped");
                sleep(Duration::from_secs(1)).await;
            }
        }

        if let Some(rabbitmq_pool) = self.rabbitmq_pool.as_ref() {
            // Close all rabbitmq connections
            // TODO what code and text to use here
            if let Err(e) = rabbitmq_pool.close(0, "shutting down").await {
                tracing::error!(
                    "Failed to close RabbitMQ connections, {}",
                    Report::from_error(e)
                );
            }
        }

        if self.shutdown.receiver_count() > 0 {
            tracing::error!("Not all tasks stopped. Exiting anyway");
        } else {
            tracing::info!("All tasks stopped, goodbye!");
        }

        Ok(())
    }
}

#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "OpenTalk Controller Frontend API",
        description = "Specifies the endpoints and structure of the OpenTalk Controller Frontend API",
    ),
    tags(
        (
            name = "api::v1::auth",
            description = "Endpoints related to authentication"
        ),
        (
            name = "api::v1::invites",
            description = "Endpoints related to meeting invites"
        ),
        (
            name = "api::v1::rooms",
            description = "Endpoints related to meeting rooms"
        ),
        (
            name = "api::v1::events",
            description = "Endpoints related to events"
        ),
        (
            name = "api::v1::events::favorites",
            description = "Endpoints related to user's favorite events"
        ),
        (
            name = "api::v1::events::instances",
            description = "Endpoints related to event instances"
        ),
        (
            name = "api::v1::events::invites",
            description = "Endpoints related to event invites"
        ),
        (
            name = "api::v1::events::shared_folder",
            description = "Endpoints related to event shared folders"
        ),
        (
            name = "api::v1::assets",
            description = "Endpoints related to file assets"
        ),
        (
            name = "api::v1::sip_configs",
            description = "Endpoints related to SIP configuration"
        ),
        (
            name = "api::v1::streaming_targets",
            description = "Endpoints related to streaming targets"
        ),
        (
            name = "api::v1::turn",
            description = "Endpoints related TURN server usage"
        ),
        (
            name = "api::v1::users",
            description = "Endpoints related to user information and management"
        ),
        (
            name = "api::signaling",
            description = "Endpoints for signaling connections in a meeting"
        ),
        (
            name = "api::internal::recording",
            description = "Internal Endpoints for recording services"
        ),
        (
            name = "api::internal::call_in",
            description = "Internal Endpoints for call-in services"
        ),
    ),
    paths(
        v1::rooms::by_id::assets::by_id::get,
        v1::rooms::by_id::assets::by_id::download::get,
        v1::rooms::by_id::assets::get,
        v1::rooms::by_id::assets::post,
        v1::rooms::by_id::assets::by_id::delete,
        v1::auth::login::get,
        v1::auth::login::post,
        v1::auth::logout::post,
        v1::events::by_id::delete,
        v1::users::me::event_favorites::by_id::put,
        v1::users::me::event_favorites::by_id::delete,
        v1::events::by_id::get,
        v1::events::get,
        v1::events::by_id::instances::by_id::get,
        v1::events::by_id::instances::get,
        v1::events::instances::get,
        v1::events::by_id::instances::by_id::patch,
        v1::events::by_id::invite::patch,
        v1::events::by_id::invites::post,
        v1::events::by_id::invite::delete,
        v1::events::by_id::invites::email::delete,
        v1::events::by_id::invites::by_id::delete,
        v1::users::me::pending_invites::get,
        v1::events::by_id::invites::get,
        v1::events::by_id::invites::email::patch,
        v1::events::by_id::invites::by_id::patch,
        v1::events::post,
        v1::events::by_id::patch,
        v1::events::by_id::shared_folder::get,
        v1::events::by_id::shared_folder::put,
        v1::events::by_id::shared_folder::delete,
        v1::invite::verify::post,
        v1::rooms::get,
        v1::rooms::by_id::delete,
        v1::rooms::by_id::get,
        v1::rooms::by_id::event::get,
        v1::rooms::by_id::start::post,
        v1::rooms::by_id::start_invited::post,
        v1::rooms::by_id::tariff::get,
        v1::rooms::post,
        v1::rooms::by_id::patch,
        v1::rooms::by_id::sip::delete,
        v1::rooms::by_id::sip::get,
        v1::rooms::by_id::sip::put,
        v1::rooms::by_id::streaming_targets::by_id::delete,
        v1::rooms::by_id::streaming_targets::by_id::get,
        v1::rooms::by_id::streaming_targets::get,
        v1::rooms::by_id::streaming_targets::by_id::patch,
        v1::rooms::by_id::streaming_targets::post,
        v1::rooms::name::verify::post,
        v1::turn::get,
        v1::users::find::get,
        v1::users::me::get,
        v1::users::me::assets::get,
        v1::users::me::tariff::get,
        v1::users::by_id::get,
        v1::users::me::patch,
        api::signaling::get,
        api::internal::assets::post_asset,
        api::internal::recording::post_start,
        api::internal::recording::get_upload,
        api::internal::call_in::post,
        api::internal::module_resources::create,
        api::internal::module_resources::get,
        api::internal::module_resources::patch,
        api::internal::module_resources::delete,
        livekit::rtc::get,
        livekit::rtc::validate::get,
        livekit::rtc::v1::get,
        livekit::rtc::v1::validate::get,
    ),
    components(
        // These schemas cannot be auto-collected by utoipa from `paths(...)`:
        // it only walks request/response *bodies*, not types used in query
        // parameters, response headers, or generic instantiations. Everything
        // else referenced from a body is registered automatically.
        schemas(
            // Response headers
            v1::response::headers::CursorLink,
            v1::response::headers::PageLink,
            // Generic instantiations (utoipa can't infer the monomorphizations)
            opentalk_types_api_v1::pagination::Cursor::<opentalk_types_api_v1::events::GetEventInstancesCursorData>,
            opentalk_types_api_v1::pagination::Cursor::<opentalk_types_api_v1::events::GetEventsCursorData>,
            opentalk_types_api_v1::pagination::Cursor::<opentalk_types_api_v1::events::GetEventsAndInstancesCursorData>,
            // Query-parameters
            opentalk_types_common::assets::AssetFileKind,
            opentalk_types_common::assets::AssetSorting,
            opentalk_types_common::assets::FileExtension,
            opentalk_types_common::order::Ordering,
            opentalk_types_common::pagination::ItemCount,
            opentalk_types_common::pagination::Page,
            opentalk_types_common::pagination::PageSize,
            // Nested types the derive does not recurse into
            opentalk_types_common::time::RecurrenceRule,
            // Path segments
            opentalk_types_common::rooms::RoomIdOrAlias,
        ),
        responses(
            crate::api::responses::BadRequest,
            crate::api::responses::BinaryData,
            crate::api::responses::InternalServerError,
            crate::api::responses::Unauthorized,
            crate::api::responses::Forbidden,
            crate::api::responses::NotFound,
        ),
    ),
    modifiers(&SecurityAddon),
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};

        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "BearerAuth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
    }
}

fn v1_scope(
    settings_provider: SettingsProvider,
    authorizer: Data<Authorizer>,
    inventory_provider: Data<dyn InventoryProvider>,
    oidc_ctx: Data<dyn OidcTokenHandler>,
    authorization: AuthorizationTransform,
) -> Scope {
    // the latest version contains the root services

    let scope = web::scope("/v1");

    scope.service(
        web::scope("")
            .wrap(authorization)
            .wrap(api::v1::middleware::user_auth::OidcAuth {
                settings_provider,
                inventory_provider,
                authorizer,
                oidc_ctx,
            })
            .service(api::signaling::get)
            .service(v1::invite::verify::post)
            .service(v1::auth::login::post)
            .service(v1::auth::login::get)
            .service(v1::auth::logout::post)
            .service(v1::rooms::by_id::roomserver::start_invited::post)
            .service(v1::rooms::by_id::start_invited::post)
            // The invite-code API has been removed; respond with `410 Gone` instead of `404 Not Found`
            .service(v1::rooms::by_id::invites::collection())
            .service(v1::rooms::by_id::invites::by_code())
            .service(v1::turn::get)
            .service(v1::rooms::by_id::assets::by_id::proxy::get)
            .service(v1::users::find::get)
            .service(v1::users::me::patch)
            .service(v1::users::me::get)
            .service(v1::users::me::tariff::get)
            .service(v1::users::me::assets::get)
            .service(v1::users::me::pending_invites::get)
            .service(v1::users::by_id::get)
            .service(v1::rooms::get)
            .service(v1::rooms::post)
            .service(v1::rooms::by_id::patch)
            .service(v1::rooms::by_id::get)
            .service(v1::rooms::by_id::event::get)
            .service(v1::rooms::by_id::tariff::get)
            .service(v1::rooms::by_id::start::post)
            .service(v1::rooms::by_id::roomserver::start::post)
            .service(v1::rooms::by_id::delete)
            .service(v1::rooms::name::verify::post)
            .service(v1::events::post)
            .service(v1::events::get)
            // "/events/instances" conflicts with "/events/{event_id}" and thus must be listed before
            .service(v1::events::instances::get)
            .service(v1::events::by_id::get)
            .service(v1::events::by_id::patch)
            .service(v1::events::by_id::delete)
            .service(v1::users::me::event_favorites::by_id::put)
            .service(v1::users::me::event_favorites::by_id::delete)
            .service(v1::events::by_id::instances::by_id::get)
            .service(v1::events::by_id::instances::get)
            .service(v1::events::by_id::instances::by_id::patch)
            .service(v1::events::by_id::invites::post)
            .service(v1::events::by_id::invites::get)
            .service(v1::events::by_id::invites::email::delete)
            .service(v1::events::by_id::invites::by_id::delete)
            .service(v1::events::by_id::invites::email::patch)
            .service(v1::events::by_id::invites::by_id::patch)
            .service(v1::events::by_id::invite::patch)
            .service(v1::events::by_id::invite::delete)
            .service(v1::events::by_id::shared_folder::get)
            .service(v1::events::by_id::shared_folder::put)
            .service(v1::events::by_id::shared_folder::delete)
            .service(v1::rooms::by_id::sip::get)
            .service(v1::rooms::by_id::sip::put)
            .service(v1::rooms::by_id::sip::delete)
            .service(v1::rooms::by_id::assets::get)
            .service(v1::rooms::by_id::assets::post)
            .service(v1::rooms::by_id::assets::by_id::get)
            .service(v1::rooms::by_id::assets::by_id::download::get)
            .service(v1::rooms::by_id::assets::by_id::delete)
            .service(v1::rooms::by_id::streaming_targets::get)
            .service(v1::rooms::by_id::streaming_targets::post)
            .service(v1::rooms::by_id::streaming_targets::by_id::get)
            .service(v1::rooms::by_id::streaming_targets::by_id::patch)
            .service(v1::rooms::by_id::streaming_targets::by_id::delete),
    )
}

fn internal_service_scope(auth_middleware: Option<ApiKeyAuthorization>) -> Scope {
    let services = web::scope("/internal");

    let Some(auth_middleware) = auth_middleware else {
        static WARN: std::sync::Once = std::sync::Once::new();
        WARN.call_once(|| {
            tracing::debug!(
                "Missing `http.service_api_keys` configuration, internal service routes are disabled"
            );
        });

        return services;
    };

    services.service(
        web::scope("")
            .wrap(auth_middleware)
            .service(api::internal::assets::post_asset)
            .service(api::internal::module_resources::create)
            .service(api::internal::module_resources::get)
            .service(api::internal::module_resources::patch)
            .service(api::internal::module_resources::delete)
            .service(api::internal::call_in::post)
            .service(api::internal::recording::post_start)
            .service(api::internal::recording::get_upload)
            .service(api::internal::transcription::post_start),
    )
}

fn livekit_scope() -> Scope {
    web::scope("livekit")
        .service(livekit::rtc::get)
        .service(livekit::rtc::v1::get)
        .service(livekit::rtc::validate::get)
        .service(livekit::rtc::v1::validate::get)
}

fn setup_cors(settings_provider: SettingsProvider) -> Cors {
    use actix_web::http::{Method, header::*};

    let settings = settings_provider.get();
    let frontend_base_url = &settings.frontend.base_url;
    let cors_allowed_origins = &settings.http.cors;

    let cors_allowed_origins = cors_allowed_origins
        .allowed_origin
        .clone()
        .map(HttpCorsAllowedOrigins::into_vec)
        .unwrap_or_else(|| {
            vec![HttpCorsAllowedOrigin::try_from_url_relaxed(frontend_base_url.clone()).unwrap()]
        });

    let mut cors = Cors::default();

    for allowed_origin in cors_allowed_origins {
        cors = if allowed_origin.is_wildcard() {
            cors.allow_any_origin()
        } else {
            cors.allowed_origin(&allowed_origin.header_value())
        }
    }

    cors.allowed_header(CONTENT_TYPE)
        .allowed_header(AUTHORIZATION)
        .expose_headers([LINK])
        .allowed_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
}

/// Set up TLS for the HTTP server that is provided by the controller
///
/// Receives the TLS-related settings from the controller configuration
/// which contains the path to the private key and the certificate files
/// from where the TLS configuration is loaded and set up.
fn setup_rustls(tls: &HttpTls) -> Result<rustls::ServerConfig> {
    let cert_file = File::open(&tls.certificate).with_whatever_context(|_| {
        format!("Failed to open certificate file {:?}", tls.certificate)
    })?;
    let certs = rustls_pemfile::certs(&mut BufReader::new(cert_file))
        .collect::<Result<Vec<CertificateDer>, _>>()
        .whatever_context("Invalid certificate")?;

    let private_key_file = File::open(&tls.private_key).with_whatever_context(|_| {
        format!(
            "Failed to open pkcs8 private key file {:?}",
            tls.private_key
        )
    })?;
    let mut key = rustls_pemfile::pkcs8_private_keys(&mut BufReader::new(private_key_file))
        .collect::<Result<Vec<PrivatePkcs8KeyDer>, _>>()
        .whatever_context("Invalid pkcs8 private key")?;

    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, rustls_pki_types::PrivateKeyDer::Pkcs8(key.remove(0)))
        .whatever_context("Invalid DER-encoded key ")?;

    Ok(config)
}

fn is_ipv6_available() -> bool {
    TcpListener::bind((Ipv6Addr::UNSPECIFIED, 0)).is_ok()
}

fn determine_socket_address(
    config_address: Option<&str>,
    config_port: u16,
) -> std::io::Result<Vec<SocketAddr>> {
    let to_socket_addrs = if let Some(addr) = config_address {
        Vec::from_iter((addr, config_port).to_socket_addrs()?)
    } else if is_ipv6_available() {
        Vec::from_iter((Ipv6Addr::UNSPECIFIED, config_port).to_socket_addrs()?)
    } else {
        Vec::from_iter((Ipv4Addr::UNSPECIFIED, config_port).to_socket_addrs()?)
    };
    Ok(to_socket_addrs)
}

pub fn load_settings_provider(optional_config_path: Option<&Path>) -> Result<SettingsProvider> {
    let settings_provider =
        SettingsProvider::load_from_path_or_standard_paths(optional_config_path)
            .whatever_context("Failed to load settings")?;

    let settings = settings_provider.get();

    if let Err(e) = trace::init(&settings.logging) {
        let report = Report::from_error(e);
        eprintln!("Could not initialize log output: {report}");
    }

    Ok(settings_provider)
}

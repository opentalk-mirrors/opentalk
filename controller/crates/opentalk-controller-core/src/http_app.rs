// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Construction of the HTTP application served by the controller.

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{
    App, Error, Scope,
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    web::{self, Data},
};
use opentalk_asset_storage::{ObjectStorage, StorageNotifier};
use opentalk_controller_api_actix_web::{v1, well_known};
use opentalk_controller_api_authorization::{
    authorization::Authorizer, middleware::AuthorizationTransform,
};
use opentalk_controller_service::{
    controller_backend::roomserver::SignalingProxyBackend,
    oidc::{Cache, OidcTokenHandler},
};
use opentalk_controller_service_facade::OpenTalkControllerService;
use opentalk_controller_settings::{
    SettingsProvider,
    common::{HttpCorsAllowedOrigin, HttpCorsAllowedOrigins},
};
use opentalk_inventory::InventoryProvider;
use opentalk_keycloak_admin::KeycloakAdminClient;
use opentalk_service_auth::service::ApiKeyAuthorization;
use tokio::sync::broadcast;
use tracing_actix_web::TracingLogger;

use crate::{
    api::{
        self, livekit,
        v1::{middleware::metrics::RequestMetrics, response::error::json_error_handler},
    },
    metrics::{self, CombinedMetrics},
    swagger::WithSwagger as _,
    trace::ReducedSpanBuilder,
};

/// Everything required to build the controller's HTTP application.
///
/// The HTTP server builds one application per worker, so this is cloned for
/// each of them. All fields are cheap to clone.
#[derive(Clone)]
pub struct HttpAppState {
    pub settings_provider: SettingsProvider,
    pub service: Arc<dyn OpenTalkControllerService>,
    /// The roomserver backend used for signaling, if the internal roomserver is used
    pub signaling: Option<Arc<dyn SignalingProxyBackend>>,
    pub oidc_cache: Arc<Cache>,
    pub oidc: Arc<dyn OidcTokenHandler>,
    pub inventory_provider: Arc<dyn InventoryProvider>,
    pub storage: Arc<ObjectStorage>,
    pub storage_notifier: Arc<dyn StorageNotifier>,
    pub user_search_client: Arc<Option<KeycloakAdminClient>>,
    pub authorizer: Authorizer,
    pub shutdown: broadcast::Sender<()>,
    pub metrics: Arc<CombinedMetrics>,
    pub http_client: reqwest::Client,
    /// Authentication for the `/internal` endpoints, which are disabled when `None`
    pub service_auth_middleware: Option<ApiKeyAuthorization>,
}

impl HttpAppState {
    /// Build the HTTP application with all middlewares and endpoints.
    pub fn build_app(
        &self,
    ) -> App<
        impl ServiceFactory<
            ServiceRequest,
            Config = (),
            Response = ServiceResponse<impl MessageBody + use<>>,
            Error = Error,
            InitError = (),
        > + use<>,
    > {
        let cors = setup_cors(self.settings_provider.clone());

        let inventory_provider = Data::from(self.inventory_provider.clone());
        let oidc_ctx = Data::from(self.oidc.clone());
        let authorizer = Data::new(self.authorizer.clone());
        let authorization = AuthorizationTransform::new(self.authorizer.clone());

        let swagger_service_enabled = !self.settings_provider.get().endpoints.disable_openapi;

        App::new()
            .wrap(RequestMetrics::new(self.metrics.endpoint.clone()))
            .wrap(cors)
            .wrap(TracingLogger::<ReducedSpanBuilder>::new())
            .wrap(api::v1::middleware::headers::Headers {})
            .app_data(Data::from(self.service.clone()))
            .app_data(Data::new(self.signaling.clone()))
            .app_data(Data::from(self.oidc_cache.clone()))
            .app_data(web::JsonConfig::default().error_handler(json_error_handler))
            .app_data(Data::new(self.settings_provider.clone()))
            .app_data(inventory_provider.clone())
            .app_data(Data::from(self.storage.clone()))
            .app_data(oidc_ctx.clone())
            .app_data(Data::from(self.user_search_client.clone()))
            .app_data(authorizer.clone())
            .app_data(Data::new(self.shutdown.clone()))
            .app_data(Data::from(self.metrics.clone()))
            .app_data(Data::new(self.http_client.clone()))
            .app_data(Data::from(self.storage_notifier.clone()))
            .service(well_known::opentalk::api::get)
            .service(metrics::metrics)
            .with_swagger_service_if(swagger_service_enabled)
            .service(internal_service_scope(self.service_auth_middleware.clone()))
            .service(livekit_scope())
            .service(v1_scope(
                self.settings_provider.clone(),
                authorizer,
                inventory_provider,
                oidc_ctx,
                authorization,
            ))
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
            // The invite-code API has been removed; respond with `410 Gone` instead of `404 Not
            // Found`
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
            // "/events/instances" conflicts with "/events/{event_id}" and thus must be listed
            // before
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
            vec![
                HttpCorsAllowedOrigin::try_from_url_relaxed(frontend_base_url.clone())
                    .expect("frontend.base_url must be a valid URL for CORS configuration"),
            ]
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

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::{
    Avatar, CallIn, Database, Defaults, Endpoints, Etcd, Frontend, Http, Logging, Metrics, MinIO,
    Monitoring, Oidc, OperatorInformation, RabbitMq, Redis, SharedFolder, Tariffs, Tenants,
    UserSearchBackend, oidc_and_user_search_builder::OidcAndUserSearchBuilder,
};
use crate::{Result, RoomServer, SettingsError, SettingsRaw, settings_file::UsersFindBehavior};

/// The settings used for the OpenTalk controller at runtime
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    /// The frontend information.
    pub frontend: Frontend,

    /// The OIDC configuration for OpenTalk.
    pub oidc: Oidc,

    /// The user search backend.
    pub user_search_backend: Option<UserSearchBackend>,

    /// The user search behavior.
    pub users_find_behavior: UsersFindBehavior,

    /// The HTTP service settings.
    pub http: Http,

    /// The database connection settings.
    pub database: Database,

    /// The redis connection settings.
    pub redis: Option<Redis>,

    /// The RabbitMQ connection settings.
    pub rabbit_mq: Option<RabbitMq>,

    /// The logging settings.
    pub logging: Logging,

    /// The avatar settings.
    pub avatar: Avatar,

    /// The metrics settings.
    pub metrics: Metrics,

    /// The etcd settings.
    pub etcd: Option<Etcd>,

    /// The SharedFolder settings.
    pub shared_folder: Option<SharedFolder>,

    /// The endpoint settings.
    pub endpoints: Endpoints,

    /// The minio settings.
    pub minio: MinIO,

    /// The monitoring settings.
    pub monitoring: Option<Monitoring>,

    /// The call-in settings.
    pub call_in: Option<CallIn>,

    /// The tenant configuration.
    pub tenants: Tenants,

    /// The tariff configuration.
    pub tariffs: Tariffs,

    /// The defaults configuration.
    pub defaults: Defaults,

    /// Information about the operator.
    pub operator_information: Option<OperatorInformation>,

    /// The roomserver configuration
    pub roomserver: RoomServer,
}

impl Settings {
    pub(crate) fn try_reload_from(&mut self, new_raw: SettingsRaw) -> Result<()> {
        let new = Settings::try_from(new_raw)?;

        // reload metrics
        self.metrics = new.metrics;

        // reload avatar
        self.avatar = new.avatar;

        // reload call-in
        self.call_in = new.call_in;

        Ok(())
    }

    /// Returns the call-in of this [`Settings`].
    pub fn call_in(&self) -> Option<&CallIn> {
        self.call_in.as_ref()
    }

    /// Returns the call-in tel of this [`Settings`].
    pub fn call_in_tel(&self) -> Option<&str> {
        self.call_in().map(|call_in| call_in.tel())
    }

    /// Returns the call-in enable phone number mapping of this [`Settings`].
    pub fn call_in_enable_phone_number_mapping(&self) -> Option<bool> {
        self.call_in().map(|call_in| call_in.enable_phone_mapping())
    }

    /// Returns the call-in mask unmapped numbers of this [`Settings`].
    pub fn call_in_mask_unmapped_numbers(&self) -> Option<bool> {
        self.call_in()
            .map(|call_in| call_in.mask_unmapped_numbers())
    }

    /// Returns the call-in default country code of this [`Settings`].
    pub fn call_in_default_country_code(&self) -> Option<phonenumber::country::Id> {
        self.call_in().map(|call_in| call_in.default_country_code())
    }
}

impl TryFrom<SettingsRaw> for Settings {
    type Error = SettingsError;

    fn try_from(raw: SettingsRaw) -> Result<Self, Self::Error> {
        // Explicitly destructure SettingsRaw to detect unused fields
        let SettingsRaw {
            frontend,
            database,
            keycloak,
            oidc: raw_oidc,
            user_search,
            http,
            redis,
            rabbit_mq,
            logging,
            avatar,
            metrics,
            etcd,
            shared_folder,
            call_in,
            defaults,
            endpoints,
            minio,
            monitoring,
            tenants,
            tariffs,
            roomserver,
            operator_information,
            // do not use the rest pattern (`..`) here!
        } = raw;

        let OidcAndUserSearchBuilder {
            oidc,
            user_search_backend,
            users_find_behavior,
        } = OidcAndUserSearchBuilder::load_from_settings_raw(
            keycloak,
            raw_oidc,
            user_search,
            endpoints.as_ref(),
        )?;

        let frontend = frontend.into();
        let http = Http::from(http);
        let database = database.into();
        let redis = redis.map(Into::into);
        let rabbit_mq = rabbit_mq.map(Into::into);
        let logging = logging.map(Into::into).unwrap_or_default();
        let avatar = avatar.map(Into::into).unwrap_or_default();
        let metrics = metrics.map(Into::into).unwrap_or_default();
        let etcd = etcd.map(Into::into);
        let shared_folder = shared_folder.map(Into::into);
        let endpoints = endpoints.map(Into::into).unwrap_or_default();
        let minio = minio.into();
        let monitoring = monitoring.map(Into::into);
        let call_in = call_in.map(Into::into);
        let tenants = tenants.map(Into::into).unwrap_or_default();
        let tariffs = tariffs.map(Into::into).unwrap_or_default();
        let defaults = defaults
            .map(TryInto::try_into)
            .transpose()?
            .unwrap_or_default();
        let operator_information = operator_information.map(Into::into);
        let roomserver = roomserver.try_into()?;

        if http.service_api_keys.is_none() {
            return Err(SettingsError::HttpServiceApiKeysMissing);
        }

        Ok(Settings {
            frontend,
            oidc,
            user_search_backend,
            users_find_behavior,
            http,
            database,
            redis,
            rabbit_mq,
            logging,
            avatar,
            metrics,
            etcd,
            shared_folder,
            endpoints,
            minio,
            monitoring,
            call_in,
            tenants,
            tariffs,
            defaults,
            operator_information,
            roomserver,
        })
    }
}

#[cfg(test)]
pub(crate) fn minimal_example() -> Settings {
    use std::{collections::BTreeSet, time::Duration};

    use openidconnect::{ClientId, ClientSecret};
    use opentalk_roomserver_modules::ECHO_MODULE_ID;
    use opentalk_roomserver_types::{
        module_settings::ModuleSettings, rate_limit::RateLimitSettings,
    };
    use opentalk_roomserver_types_livekit::LiveKitSettings;
    use opentalk_service_auth::{ApiKey, service::ApiKeys};
    use opentalk_types_common::time::TimeZone;
    use url::Url;

    use super::OidcController;
    use crate::{
        DEFAULT_LIBRAVATAR_URL, DEFAULT_STATIC_TARIFF_NAME, DEFAULT_STATIC_TENANT_ID, Frontend,
        OidcFrontend, RoomServerKind, TariffAssignment, TenantAssignment,
        settings_file::RoomAlias,
        settings_runtime::{
            HttpCors,
            database::DEFAULT_DATABASE_MAX_CONNECTIONS,
            defaults::default_user_language,
            http::{DEFAULT_HTTP_PORT, DEFAULT_UPLOAD_SIZE_LIMIT},
        },
    };

    let mut modules = ModuleSettings::new();
    modules.insert_empty(ECHO_MODULE_ID);
    modules
        .insert(&LiveKitSettings {
            api_key: "devkey".to_string(),
            api_secret: "secret".to_string(),
            public_url: "ws://localhost:7880".to_string(),
            service_url: "http://localhost:7880".parse().expect("URL must be valid"),
        })
        .expect("LiveKitSettings must be valid");

    Settings {
        frontend: Frontend {
            base_url: Url::parse("https://example.com").unwrap(),
        },
        oidc: Oidc {
            controller: OidcController {
                authority: "http://localhost:8080/realms/opentalk"
                    .parse()
                    .expect("must be a valid url"),
                client_id: ClientId::new("Controller".to_string()),
                client_secret: ClientSecret::new("mysecret".to_string()),
            },
            frontend: OidcFrontend {
                authority: "http://localhost:8080/realms/opentalk"
                    .parse()
                    .expect("must be a valid url"),
                client_id: ClientId::new("Webapp".to_string()),
            },
        },
        user_search_backend: None,
        users_find_behavior: UsersFindBehavior::Disabled,
        http: Http {
            addr: None,
            port: DEFAULT_HTTP_PORT,
            tls: None,
            cors: HttpCors::default(),
            service_api_keys: Some(ApiKeys::new(vec![ApiKey::new("controller", "secret")])),
            upload_size_limit: DEFAULT_UPLOAD_SIZE_LIMIT,
        },
        database: Database {
            url: "postgres://postgres:password123@localhost:5432/opentalk".to_string(),
            max_connections: DEFAULT_DATABASE_MAX_CONNECTIONS,
        },
        redis: None,
        rabbit_mq: None,
        logging: Logging {
            default_directives: None,
            otlp_tracing: None,
        },
        avatar: Avatar {
            libravatar_url: DEFAULT_LIBRAVATAR_URL.to_string(),
        },
        metrics: Metrics { allowlist: vec![] },
        etcd: None,
        shared_folder: None,
        endpoints: Endpoints {
            event_invite_external_email_address: false,
            disallow_custom_display_name: false,
            disable_openapi: false,
        },
        minio: MinIO {
            uri: "http://localhost:9555"
                .parse()
                .expect("must be a valid url"),
            bucket: "controller".to_string(),
            access_key: "minioadmin".to_string(),
            secret_key: "minioadmin".to_string(),
            region: None,
            force_path_style: None,
        },
        monitoring: None,
        call_in: None,
        tenants: Tenants {
            assignment: TenantAssignment::Static {
                static_tenant_id: DEFAULT_STATIC_TENANT_ID.to_string(),
            },
        },
        tariffs: Tariffs {
            assignment: TariffAssignment::Static {
                static_tariff_name: DEFAULT_STATIC_TARIFF_NAME.to_string(),
            },
        },
        defaults: Defaults {
            user_language: default_user_language(),
            timezone: TimeZone::default(),
            disabled_features: BTreeSet::new(),
            room_alias: RoomAlias::default(),
        },
        operator_information: None,
        roomserver: RoomServer {
            kind: RoomServerKind::External {
                service_url: "http://localhost:11333"
                    .parse()
                    .expect("must be a valid url"),
                api_key: ApiKey::new("roomserver", "secret"),
            },
            modules,
            websocket_rate_limit: Some(RateLimitSettings {
                tokens_per_second: 10,
                token_bucket_size: 30,
            }),
            room_idle_timeout: Duration::from_secs(60),
        },
    }
}

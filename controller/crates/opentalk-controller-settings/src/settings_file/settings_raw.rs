// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

use super::{
    Avatar, CallIn, Database, Defaults, Endpoints, Etcd, Frontend, Http, Keycloak, Logging,
    Metrics, MinIO, MonitoringSettings, Oidc, OperatorInformation, RabbitMqConfig, RedisConfig,
    SharedFolder, Tariffs, Tenants, UserSearch,
};
use crate::settings_file::RoomServer;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SettingsRaw {
    pub(crate) frontend: Frontend,

    pub(crate) database: Database,

    #[serde(default)]
    pub(crate) keycloak: Option<Keycloak>,

    #[serde(default)]
    pub(crate) oidc: Option<Oidc>,

    #[serde(default)]
    pub(crate) user_search: Option<UserSearch>,

    #[serde(default)]
    pub(crate) http: Option<Http>,

    #[serde(default)]
    pub(crate) redis: Option<RedisConfig>,

    #[serde(default)]
    pub(crate) rabbit_mq: Option<RabbitMqConfig>,

    #[serde(default)]
    pub(crate) logging: Option<Logging>,

    #[serde(default)]
    pub(crate) avatar: Option<Avatar>,

    #[serde(default)]
    pub(crate) metrics: Option<Metrics>,

    #[serde(default)]
    pub(crate) etcd: Option<Etcd>,

    #[serde(default)]
    pub(crate) shared_folder: Option<SharedFolder>,

    #[serde(default)]
    pub(crate) call_in: Option<CallIn>,

    #[serde(default)]
    pub(crate) defaults: Option<Defaults>,

    #[serde(default)]
    pub(crate) endpoints: Option<Endpoints>,

    pub(crate) minio: MinIO,

    #[serde(default)]
    pub(crate) monitoring: Option<MonitoringSettings>,

    #[serde(default)]
    pub(crate) tenants: Option<Tenants>,

    #[serde(default)]
    pub(crate) tariffs: Option<Tariffs>,

    pub(crate) roomserver: RoomServer,

    #[serde(default)]
    pub(crate) operator_information: Option<OperatorInformation>,
}

#[cfg(any(feature = "test-util", test))]
pub(crate) fn settings_raw_minimal_example() -> SettingsRaw {
    use openidconnect::{ClientId, ClientSecret};
    use opentalk_roomserver_modules::ECHO_MODULE_ID;
    use opentalk_roomserver_types::module_settings::ModuleSettings;
    use opentalk_roomserver_types_livekit::LiveKitSettings;
    use opentalk_service_auth::{ApiKey, service::ApiKeys};
    use url::Url;

    use super::{Http, OidcController, OidcFrontend};
    use crate::settings_file::RoomServerKind;

    // Keep this in sync with `SETTINGS_RAW_MINIMAL_CONFIG_TOML` below: the
    // `livekit` and `echo` modules are mandatory at the runtime conversion
    // step (see `settings_runtime::roomserver::MANDATORY_MODULES`), and the
    // HTTP service API keys are mandatory as well.
    let mut modules = ModuleSettings::new();
    modules.insert_empty(ECHO_MODULE_ID);
    modules
        .insert(&LiveKitSettings {
            api_key: "devkey".to_string(),
            api_secret: "secret".to_string(),
            public_url: "ws://localhost:7880".to_string(),
            service_url: "http://localhost:7880/".parse().expect("URL must be valid"),
        })
        .expect("LiveKitSettings must be valid");

    SettingsRaw {
        frontend: Frontend {
            base_url: Url::parse("https://example.com").unwrap(),
        },
        database: Database {
            url: "postgres://postgres:password123@localhost:5432/opentalk".to_string(),
            max_connections: None,
        },
        keycloak: None,
        oidc: Some(Oidc {
            authority: "http://localhost:8080/realms/opentalk"
                .parse()
                .expect("must be a valid url"),
            frontend: OidcFrontend {
                authority: None,
                client_id: ClientId::new("Webapp".to_string()),
            },
            controller: OidcController {
                authority: None,
                client_id: ClientId::new("Controller".to_string()),
                client_secret: ClientSecret::new("mysecret".to_string()),
            },
        }),
        user_search: Some(UserSearch {
            backend: None,
            users_find_behavior: None,
        }),
        http: Some(Http {
            service_api_keys: Some(ApiKeys::new(vec![ApiKey::new("controller", "secret")])),
            ..Default::default()
        }),
        redis: None,
        rabbit_mq: None,
        logging: None,
        avatar: None,
        metrics: None,
        etcd: None,
        shared_folder: None,
        call_in: None,
        defaults: None,
        endpoints: None,
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
        tenants: None,
        tariffs: None,
        roomserver: RoomServer {
            kind: RoomServerKind::External {
                service_url: "http://localhost:11333"
                    .parse()
                    .expect("must be a valid url"),
                api_key: ApiKey::new("roomserver", "secret"),
            },
            modules,
            websocket_rate_limit: None,
            room_idle_timeout: Some(60),
        },
        operator_information: None,
    }
}

#[cfg(test)]
pub(crate) const SETTINGS_RAW_MINIMAL_CONFIG_TOML: &str = r#"
        [http]
        service_api_keys = [{ "id" = "controller", "secret" = "secret" }]

        [frontend]
        base_url = "https://example.com"

        [database]
        url = "postgres://postgres:password123@localhost:5432/opentalk"

        [minio]
        uri = "http://localhost:9555"
        bucket = "controller"
        access_key = "minioadmin"
        secret_key = "minioadmin"

        [oidc]
        authority = "http://localhost:8080/realms/opentalk"

        [oidc.frontend]
        client_id = "Webapp"

        [oidc.controller]
        client_id = "Controller"
        client_secret = "mysecret"

        [roomserver]
        kind = "external"
        service_url = "http://localhost:11333"
        api_key = { id = "roomserver", secret = "secret" }
        [roomserver.modules]
        [roomserver.modules.livekit]
        public_url = "ws://localhost:7880"
        service_url = "http://localhost:7880/"
        api_key = "devkey"
        api_secret = "secret"
        [roomserver.modules.echo]
        "#;

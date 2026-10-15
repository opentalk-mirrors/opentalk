// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Runs the controller HTTP application with mocked backends on a local port.
//!
//! This is the only module of the HTTP tests that knows which web framework is used. The tests
//! themselves interact with the application exclusively through HTTP.
//!
//! Unexpected calls to a mock panic inside the server, which aborts the connection and thereby
//! fails the request in the test. Expected calls are verified through the HTTP response.

use std::{
    net::SocketAddr,
    sync::{Arc, OnceLock},
    thread::JoinHandle,
};

use actix_web::{HttpServer, dev::ServerHandle};
use chrono::Utc;
use openidconnect::AccessToken;
use opentalk_asset_storage::{NoOpStorageNotifier, ObjectStorage};
use opentalk_controller_api_authorization::authorization::{Authorizer, MockAuthorizerBackend};
use opentalk_controller_core::{CombinedMetrics, HttpAppState};
use opentalk_controller_service::oidc::{Cache, MockOidcTokenHandler};
use opentalk_controller_service_facade::MockOpenTalkControllerService;
use opentalk_controller_settings::{
    SettingsProvider, test_util::settings_provider_from_example_raw_settings,
};
use opentalk_inventory::{MockInventoryProvider, Tenant, User};
use opentalk_types_common::utils::ExampleData as _;
use tokio::sync::broadcast;

/// The access token that authenticates the user passed to [`TestApp::authenticate`].
pub const ACCESS_TOKEN: &str = "test-access-token";

/// The controller HTTP application with mocked backends, before it is started.
///
/// Set the expectations on the mocks, then call [`TestApp::start`].
pub struct TestApp {
    pub service: MockOpenTalkControllerService,
    pub authorizer: MockAuthorizerBackend,
    pub oidc: MockOidcTokenHandler,
    pub inventory: MockInventoryProvider,
    settings_provider: SettingsProvider,
    oidc_cache: Arc<Cache>,
}

impl TestApp {
    pub fn new() -> Self {
        Self {
            service: MockOpenTalkControllerService::new(),
            authorizer: MockAuthorizerBackend::new(),
            oidc: MockOidcTokenHandler::new(),
            inventory: MockInventoryProvider::new(),
            settings_provider: settings_provider_from_example_raw_settings(),
            oidc_cache: Arc::new(Cache::create(None)),
        }
    }

    /// Let requests with [`ACCESS_TOKEN`] authenticate as `user`.
    ///
    /// The user is stored in the access token cache, so neither the OIDC provider nor the
    /// inventory is asked to verify the token.
    pub async fn authenticate(&self, user: User) {
        self.oidc_cache
            .insert_access_token(
                &AccessToken::new(ACCESS_TOKEN.to_owned()),
                Ok((Tenant::example_data(), user)),
                Some(Utc::now() + chrono::Duration::hours(1)),
            )
            .await
            .expect("access token must be cached");
    }

    /// Start serving the application on a free local port.
    pub fn start(self) -> TestServer {
        let settings = self.settings_provider.get();
        let service_auth_middleware = settings
            .http
            .service_api_keys
            .clone()
            .map(|keys| keys.auth_middleware())
            .transpose()
            .expect("service api keys must be valid");

        let state = HttpAppState {
            settings_provider: self.settings_provider,
            service: Arc::new(self.service),
            signaling: None,
            oidc_cache: self.oidc_cache,
            oidc: Arc::new(self.oidc),
            inventory_provider: Arc::new(self.inventory),
            storage: Arc::new(ObjectStorage::broken()),
            storage_notifier: Arc::new(NoOpStorageNotifier),
            user_search_client: Arc::new(None),
            authorizer: Authorizer::new(self.authorizer),
            shutdown: broadcast::channel(1).0,
            metrics: metrics(&settings.roomserver.kind),
            http_client: reqwest::Client::new(),
            service_auth_middleware,
        };

        TestServer::start(state)
    }
}

/// The metrics are registered globally, so they can only be initialized once per process.
fn metrics(roomserver_kind: &opentalk_controller_settings::RoomServerKind) -> Arc<CombinedMetrics> {
    static METRICS: OnceLock<Arc<CombinedMetrics>> = OnceLock::new();

    METRICS
        .get_or_init(|| {
            Arc::new(
                CombinedMetrics::try_init(roomserver_kind).expect("metrics must be initialized"),
            )
        })
        .clone()
}

/// A running instance of the controller HTTP application.
///
/// The server is stopped when this is dropped.
pub struct TestServer {
    addr: SocketAddr,
    handle: ServerHandle,
    thread: Option<JoinHandle<()>>,
    client: reqwest::Client,
}

impl TestServer {
    fn start(state: HttpAppState) -> Self {
        let (started_tx, started_rx) = std::sync::mpsc::channel();

        let thread = std::thread::spawn(move || {
            actix_web::rt::System::new().block_on(async move {
                let server = HttpServer::new(move || state.build_app())
                    .workers(1)
                    .disable_signals()
                    .bind(("127.0.0.1", 0))
                    .expect("test server must bind to a local port");
                let addr = server.addrs()[0];

                let server = server.run();
                started_tx
                    .send((addr, server.handle()))
                    .expect("test must wait for the server to start");

                server.await.expect("test server must run");
            });
        });

        let (addr, handle) = started_rx.recv().expect("test server must start");

        Self {
            addr,
            handle,
            thread: Some(thread),
            client: reqwest::Client::new(),
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.addr)
    }

    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.get(self.url(path))
    }

    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.post(self.url(path))
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        // The stop command is sent immediately, the returned future only awaits its completion.
        drop(self.handle.stop(false));

        if let Some(thread) = self.thread.take()
            && let Err(panic) = thread.join()
            && !std::thread::panicking()
        {
            std::panic::resume_unwind(panic);
        }
    }
}

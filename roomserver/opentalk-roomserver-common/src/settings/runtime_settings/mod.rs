// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::sync::Arc;

use http::Http;
use internal::Internal;
use opentalk_orchestrator_client::OrchestratorConfig;
use opentalk_service_auth::{ApiKey, service::ApiKeys};
use task::Task;
use telemetry::{Metrics, Monitoring, Tracing};
use url::Url;

use super::{controller_settings::ControllerConfig, settings_file::SettingsFile};

pub mod conference;
pub mod defaults;
pub mod http;
pub mod internal;
pub mod reports;
pub mod reports_typst;
pub mod task;
pub mod telemetry;

#[derive(Debug, Clone)]
pub struct Settings {
    /// HTTP web server settings
    pub http: Http,

    pub controller: Option<ControllerConfig>,

    pub orchestrator: Option<OrchestratorConfig>,

    pub monitoring: Option<Monitoring>,

    pub metrics: Option<Metrics>,

    pub tracing: Option<Tracing>,

    pub internal: Internal,

    pub task: Arc<Task>,
}

impl Settings {
    /// Creates settings for testing
    ///
    /// Do not use in production
    pub fn test_settings(api_token: String) -> Settings {
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
                api_keys: ApiKeys::new(vec![ApiKey::new("roomserver", api_token)]),
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
}

impl TryFrom<SettingsFile> for Settings {
    type Error = anyhow::Error;

    fn try_from(value: SettingsFile) -> Result<Self, Self::Error> {
        Ok(Settings {
            http: value.http.into(),
            controller: value.controller,
            orchestrator: value.orchestrator,
            monitoring: value.monitoring.map(Into::into),
            metrics: value.metrics.map(Into::into),
            tracing: value.tracing.map(Into::into),
            internal: value.internal.map(Into::into).unwrap_or_default(),
            task: Arc::new(value.task.into()),
        })
    }
}

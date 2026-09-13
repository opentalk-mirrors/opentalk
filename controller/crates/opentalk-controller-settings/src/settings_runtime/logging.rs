// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use uuid::Uuid;

use super::LoggingOltpTracing;
use crate::settings_file;

const DEFAULT_SERVICE_NAME: &str = "controller";
const DEFAULT_SERVICE_NAMESPACE: &str = "opentalk";

/// Logging configuration.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Logging {
    /// The default directives in RUST_LOG format.
    pub default_directives: Option<Vec<String>>,

    /// OTLP tracing configuration, the endpoint will only be enabled if this is set.
    pub otlp_tracing: Option<LoggingOltpTracing>,
}

impl From<settings_file::Logging> for Logging {
    fn from(
        settings_file::Logging {
            default_directives,
            otlp_tracing_endpoint,
            service_name,
            service_namespace,
            service_instance_id,
        }: settings_file::Logging,
    ) -> Self {
        let default_directives = default_directives.filter(|v| !v.is_empty());
        let otlp_tracing = otlp_tracing_endpoint.map(|endpoint| LoggingOltpTracing {
            endpoint,
            service_name: service_name.unwrap_or_else(|| DEFAULT_SERVICE_NAME.to_string()),
            service_namespace: service_namespace
                .unwrap_or_else(|| DEFAULT_SERVICE_NAMESPACE.to_string()),
            service_instance_id: service_instance_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        });
        Self {
            default_directives,
            otlp_tracing,
        }
    }
}

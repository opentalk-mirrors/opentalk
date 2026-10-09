// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

/// The logging and tracing configuration for the orchestrator
#[derive(Default, Debug, Clone, Deserialize)]
pub(crate) struct Logging {
    /// The logging directives to use for the orchestrator. Has the same format as the `RUST_LOG`
    /// environment variable.
    pub(crate) directives: Option<Vec<String>>,

    /// OTLP exporter endpoint. When unset, trace exporting is disabled.
    pub(crate) otlp_tracing_endpoint: Option<String>,

    /// Overrides the reported OpenTelemetry `service.name`.
    pub(crate) service_name: Option<String>,

    /// Overrides the reported OpenTelemetry `service.namespace`.
    pub(crate) service_namespace: Option<String>,

    /// Overrides the reported OpenTelemetry `service.instance.id`.
    pub(crate) service_instance_id: Option<String>,
}

impl Logging {
    /// Map the orchestrator logging configuration onto the shared tracing settings.
    pub(crate) fn to_tracing_settings(&self) -> opentalk_tracing::TracingSettings {
        opentalk_tracing::TracingSettings {
            default_directives: self.directives.clone(),
            otlp_tracing_endpoint: self.otlp_tracing_endpoint.clone(),
            service_name: self.service_name.clone(),
            service_namespace: self.service_namespace.clone(),
            service_instance_id: self.service_instance_id.clone(),
        }
    }
}

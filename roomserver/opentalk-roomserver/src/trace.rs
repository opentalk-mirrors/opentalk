// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Context;
use opentalk_roomserver_common::settings::Tracing;
use opentelemetry::{KeyValue, trace::TracerProvider as _};
use opentelemetry_otlp::{SpanExporter, WithExportConfig as _};
use opentelemetry_sdk::{
    Resource,
    trace::{SdkTracerProvider, Tracer},
};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{
    EnvFilter, Registry,
    fmt::Layer,
    layer::{Layered, SubscriberExt},
    util::SubscriberInitExt,
};
use uuid::Uuid;

const DEFAULT_LOGGING_DIRECTIVES: &str =
    "warn,opentalk_roomserver=info,opentalk_orchestrator_client=info";

pub fn init(settings: Option<&Tracing>) -> anyhow::Result<()> {
    // Layer which acts as filter of traces and spans.
    let filter = create_filter(settings.and_then(Tracing::log_filter));

    // FMT layer prints the trace events into stdout
    let fmt = tracing_subscriber::fmt::Layer::default();
    let tracing_layer = init_tracing_layer(settings)?;

    // Create registry which contains all layers
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt)
        .with(tracing_layer)
        .init();

    Ok(())
}

type SubscriberLayer = Layer<Layered<EnvFilter, Registry>>;
type Subscriber = Layered<EnvFilter, Registry>;

fn init_tracing_layer(
    settings: Option<&Tracing>,
) -> anyhow::Result<Option<OpenTelemetryLayer<Layered<SubscriberLayer, Subscriber>, Tracer>>> {
    match settings {
        Some(settings) => {
            let otlp_exporter = SpanExporter::builder()
                .with_tonic()
                .with_endpoint(&settings.otlp_tracing_endpoint)
                .build()
                .context("Failed to build OpenTelemetry (exporter)")?;
            let service_name = settings
                .service_name
                .clone()
                .unwrap_or_else(|| "roomserver".into());

            let service_namespace = settings
                .service_namespace
                .clone()
                .unwrap_or_else(|| "opentalk".into());

            let service_instance_id = settings
                .service_instance_id
                .clone()
                .unwrap_or_else(|| Uuid::new_v4().to_string());

            let resource = Resource::builder()
                .with_service_name(service_name)
                .with_attribute(KeyValue::new("service.namespace", service_namespace))
                .with_attribute(KeyValue::new("service.instance.id", service_instance_id))
                .with_attribute(KeyValue::new(
                    "service.version",
                    option_env!("VERGEN_GIT_SEMVER")
                        .or(option_env!("CARGO_PKG_VERSION"))
                        .unwrap_or("unknown"),
                ))
                .build();
            let tracer_provider = SdkTracerProvider::builder()
                .with_batch_exporter(otlp_exporter)
                .with_resource(resource)
                .build();

            let tracer = tracer_provider.tracer("tracing-otel-subscriber");
            Ok(Some(OpenTelemetryLayer::new(tracer)))
        }
        None => Ok(None),
    }
}

/// Create the logging filter
///
/// The priority of the different config options is `ROOMSERVER_LOG` > `RUST_LOG` > hard-coded
/// defaults.
fn create_filter(log_filter_settings: Option<String>) -> EnvFilter {
    fn read_env_var(var: &str) -> Option<String> {
        std::env::var(var).ok().filter(|v| !v.is_empty())
    }

    let directives = read_env_var("ROOMSERVER_LOG")
        .or_else(|| read_env_var(EnvFilter::DEFAULT_ENV))
        .or(log_filter_settings)
        .unwrap_or(DEFAULT_LOGGING_DIRECTIVES.to_owned());

    EnvFilter::new(directives)
}

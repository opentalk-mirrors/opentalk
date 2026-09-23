// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Shared tracing and logging initialization for OpenTalk services.
//!
//! This crate consolidates the OpenTelemetry OTLP tracer setup and the
//! `tracing-subscriber` registry configuration that was previously duplicated
//! across the controller, roomserver and recorder services.

use std::sync::Once;

use anyhow::Context as _;
use opentelemetry::{KeyValue, trace::TracerProvider as _};
use opentelemetry_otlp::{SpanExporter, WithExportConfig as _};
use opentelemetry_sdk::Resource;
pub use opentelemetry_sdk::trace::SdkTracerProvider;
use serde::Deserialize;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt as _, util::SubscriberInitExt as _};
use uuid::Uuid;

pub mod propagation;

/// Resolved configuration for exporting traces to an OTLP collector.
struct OtlpConfig {
    endpoint: String,
    service_name: String,
    service_namespace: String,
    service_instance_id: String,
    service_version: String,
}

impl OtlpConfig {
    /// Apply the shared OpenTalk defaults for the namespace (`opentalk`) and a
    /// random instance id when they are not set.
    fn new(
        endpoint: impl Into<String>,
        service_name: impl Into<String>,
        service_namespace: Option<String>,
        service_instance_id: Option<String>,
        service_version: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            service_name: service_name.into(),
            service_namespace: service_namespace.unwrap_or_else(|| "opentalk".to_owned()),
            service_instance_id: service_instance_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            service_version: service_version.into(),
        }
    }
}

/// Deserializable tracing configuration shared by the OpenTalk services.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct TracingSettings {
    /// Logging directives applied when no environment override is present.
    pub default_directives: Option<Vec<String>>,

    /// OTLP exporter endpoint. When unset, trace exporting is disabled.
    pub otlp_tracing_endpoint: Option<String>,

    /// Overrides the reported `service.name`.
    pub service_name: Option<String>,

    /// Overrides the reported `service.namespace`.
    pub service_namespace: Option<String>,

    /// Overrides the reported `service.instance.id`.
    pub service_instance_id: Option<String>,
}

impl TracingSettings {
    /// The comma-joined logging directives from the configuration, if any.
    fn filter_directives(&self) -> Option<String> {
        self.default_directives
            .as_ref()
            .map(|directives| directives.join(","))
    }

    /// Build the [`OtlpConfig`] when an endpoint is configured, falling back to
    /// `default_service_name` for the reported service name.
    fn otlp_config(
        &self,
        default_service_name: &str,
        service_version: impl Into<String>,
    ) -> Option<OtlpConfig> {
        self.otlp_tracing_endpoint.clone().map(|endpoint| {
            OtlpConfig::new(
                endpoint,
                self.service_name
                    .clone()
                    .unwrap_or_else(|| default_service_name.to_owned()),
                self.service_namespace.clone(),
                self.service_instance_id.clone(),
                service_version,
            )
        })
    }
}

/// Static, per-service tracing parameters.
///
/// Unlike [`TracingSettings`], which is loaded from the configuration file,
/// these values are compile-time constants provided by each service.
pub struct TracingParams<'a> {
    /// Default `service.name`, used when the configuration does not override it.
    pub service_name: &'a str,

    /// Service version reported to the collector.
    pub service_version: &'a str,

    /// Fallback log directives, used when neither the environment nor the
    /// configuration provide any.
    pub default_directives: &'a str,

    /// Environment variables checked, in order, for log directives before
    /// falling back to `RUST_LOG`. The first non-empty value wins.
    pub filter_env_vars: &'a [&'a str],
}

static INIT: Once = Once::new();

/// Initialize the global tracing subscriber for a service.
///
/// Installs an [`EnvFilter`], a formatting layer writing to stdout and, when
/// `settings` provide an OTLP endpoint, an OpenTelemetry OTLP export layer.
/// Returns the created [`SdkTracerProvider`], which must be passed to
/// [`shutdown`] during graceful shutdown to flush pending spans.
pub fn init(
    bootstrap: Bootstrap,
    settings: Option<&TracingSettings>,
    params: TracingParams<'_>,
) -> anyhow::Result<Option<SdkTracerProvider>> {
    let config_directives = settings.and_then(TracingSettings::filter_directives);
    let filter = create_filter(
        params.filter_env_vars,
        config_directives.as_deref(),
        params.default_directives,
    );
    let fmt = tracing_subscriber::fmt::layer();

    // conditionally add the OpenTelemetry exporter
    let (otel_layer, provider) = settings
        .and_then(|settings| settings.otlp_config(params.service_name, params.service_version))
        .map(build_tracer_provider)
        .transpose()?
        .map(|provider| {
            let tracer = provider.tracer("tracing-otel-subscriber");
            let layer = tracing_opentelemetry::layer().with_tracer(tracer);
            (layer, provider)
        })
        .unzip();

    // Tear down the bootstrap subscriber right before installing the global one.
    drop(bootstrap);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt)
        .with(otel_layer)
        .init();

    // Parse incoming traceparent/tracestate only when OTLP export is enabled.
    if provider.is_some() {
        INIT.call_once(propagation::init);
    }

    Ok(provider)
}

/// Guard for the temporary bootstrap subscriber installed by [`bootstrap`].
///
/// It must be passed to [`init`], which drops it before installing the global
/// subscriber. Requiring it as an argument makes it impossible to install the
/// global subscriber without first tearing down the bootstrap one.
#[must_use = "the bootstrap guard must be passed to `init`"]
pub struct Bootstrap {
    _guard: tracing::subscriber::DefaultGuard,
}

/// Install a temporary subscriber as the current thread's default, bridging the
/// window before [`init`] can run.
///
/// [`init`] needs the loaded configuration to resolve its directives and OTLP
/// endpoint, so it can only run after the config has been read. Events emitted
/// during that early window would otherwise be dropped, because no subscriber is
/// set yet. This bootstrap subscriber captures them, resolving directives from
/// `filter_env_vars` and `default_directives` only, since the configuration is
/// not available yet (and therefore without OTLP export).
///
/// The returned [`Bootstrap`] guard must be passed to [`init`], which tears the
/// bootstrap subscriber down before installing the global one.
pub fn bootstrap(filter_env_vars: &[&str], default_directives: &str) -> Bootstrap {
    let filter = create_filter(filter_env_vars, None, default_directives);
    let fmt = tracing_subscriber::fmt::layer();

    let subscriber = tracing_subscriber::registry().with(filter).with(fmt);

    // Install only the thread-local dispatcher, not the `log` bridge: `init`
    // installs that once, and `log::set_logger` would panic if called twice.
    let guard = tracing::dispatcher::set_default(&subscriber.into());

    Bootstrap { _guard: guard }
}

/// Flush and shut down the OTLP tracer provider, exporting any spans still
/// buffered by the batch processor.
///
/// This must be called explicitly during graceful shutdown: the global
/// subscriber installed by [`init`] keeps its own reference to the provider, so
/// dropping the handle returned by [`init`] does not trigger a flush. Passing
/// `None` (trace export disabled) is a no-op.
pub fn shutdown(provider: Option<SdkTracerProvider>) {
    let Some(provider) = provider else {
        return;
    };
    if let Err(error) = provider.shutdown() {
        eprintln!("Failed to flush OpenTelemetry spans on shutdown: {error}");
    }
}

fn build_tracer_provider(otlp: OtlpConfig) -> anyhow::Result<SdkTracerProvider> {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&otlp.endpoint)
        .build()
        .context("Failed to build OpenTelemetry exporter")?;

    let resource = Resource::builder()
        .with_service_name(otlp.service_name)
        .with_attribute(KeyValue::new("service.namespace", otlp.service_namespace))
        .with_attribute(KeyValue::new(
            "service.instance.id",
            otlp.service_instance_id,
        ))
        .with_attribute(KeyValue::new("service.version", otlp.service_version))
        .build();

    Ok(SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build())
}

/// Resolve the logging directives with override precedence:
/// environment variables > configuration file > hard-coded defaults.
fn create_filter(
    env_vars: &[&str],
    config_directives: Option<&str>,
    default_directives: &str,
) -> EnvFilter {
    let directives = env_vars
        .iter()
        .copied()
        .chain(std::iter::once(EnvFilter::DEFAULT_ENV))
        .find_map(read_env_var)
        .or_else(|| {
            config_directives
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| default_directives.to_owned());

    EnvFilter::new(directives)
}

fn read_env_var(var: &str) -> Option<String> {
    std::env::var(var).ok().filter(|value| !value.is_empty())
}

/// Expands to the current service version, preferring the git-derived
/// `VERGEN_GIT_SEMVER` over the crate's `CARGO_PKG_VERSION`.
///
/// The macro must be invoked from the service crate so the compile-time
/// environment variables resolve to that crate's build.
#[macro_export]
macro_rules! service_version {
    () => {
        ::core::option_env!("VERGEN_GIT_SEMVER")
            .or(::core::option_env!("CARGO_PKG_VERSION"))
            .unwrap_or("unknown")
    };
}

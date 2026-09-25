// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_tracing::{SdkTracerProvider, TracingParams};
use tracing::subscriber::DefaultGuard;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::settings::logging::Logging;

const DEFAULT_LOGGING_DIRECTIVES: &str = "warn,opentalk_orchestrator=info";

/// Initializes a primitive logging that can be used before the full tracing configuration is
/// loaded.
///
/// The returned guard must be dropped before the actual logging configuration is loaded.
pub fn early_logging() -> DefaultGuard {
    let fmt = tracing_subscriber::fmt::Layer::default();
    tracing_subscriber::registry()
        .with(fmt)
        .with(early_filter())
        .set_default()
}

/// Initializes logging based on the provided settings and environment variables.
///
/// Returns the OTLP tracer provider when trace export is configured, so the caller can keep it
/// alive to flush pending spans on shutdown.
pub fn init_logging(settings: Option<&Logging>) -> anyhow::Result<Option<SdkTracerProvider>> {
    let tracing_settings = settings.map(Logging::to_tracing_settings);
    opentalk_tracing::init(
        tracing_settings.as_ref(),
        TracingParams {
            service_name: "orchestrator",
            service_version: opentalk_tracing::service_version!(),
            default_directives: DEFAULT_LOGGING_DIRECTIVES,
            filter_env_vars: &["ORCHESTRATOR_LOG"],
        },
    )
}

/// Create the early logging filter.
///
/// The priority of the different config options is:
///
/// `ORCHESTRATOR_LOG` > `RUST_LOG` > [`DEFAULT_LOGGING_DIRECTIVES`]
fn early_filter() -> EnvFilter {
    fn read_env_var(var: &str) -> Option<String> {
        std::env::var(var).ok().filter(|v| !v.is_empty())
    }

    let directives = read_env_var("ORCHESTRATOR_LOG")
        .or_else(|| read_env_var(EnvFilter::DEFAULT_ENV))
        .unwrap_or(DEFAULT_LOGGING_DIRECTIVES.to_owned());

    EnvFilter::new(directives)
}

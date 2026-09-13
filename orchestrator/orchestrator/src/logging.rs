// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

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
        .with(create_filter(None))
        .set_default()
}

/// Initializes logging based on the provided settings and environment variables.
pub fn init_logging(settings: Option<&Logging>) {
    let filter = create_filter(settings.and_then(Logging::directives));

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(filter)
        .finish();

    tracing::dispatcher::set_global_default(subscriber.into())
        .expect("Failed to set global default logger");
}

/// Create the logging filter
///
/// The priority of the different config options is:
///
/// `ORCHESTRATOR_LOG` > `RUST_LOG` > settings > [`DEFAULT_LOGGING_DIRECTIVES`]
fn create_filter(log_filter_settings: Option<String>) -> EnvFilter {
    fn read_env_var(var: &str) -> Option<String> {
        std::env::var(var).ok().filter(|v| !v.is_empty())
    }

    let directives = read_env_var("ORCHESTRATOR_LOG")
        .or_else(|| read_env_var(EnvFilter::DEFAULT_ENV))
        .or(log_filter_settings)
        .unwrap_or(DEFAULT_LOGGING_DIRECTIVES.to_owned());

    EnvFilter::new(directives)
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_tracing::{Bootstrap, SdkTracerProvider, TracingParams};

use crate::settings::logging::Logging;

/// Environment variable used to configure the log filter, taking precedence over `RUST_LOG`.
const LOG_ENV: &str = "ORCHESTRATOR_LOG";

/// Default log directives, applied when neither `RUST_LOG` nor the configuration provide any.
const DEFAULT_LOGGING_DIRECTIVES: &str = "warn,opentalk_orchestrator=info";

/// Initializes the global tracing subscriber.
///
/// The log filter directives are resolved with the following precedence:
/// `ORCHESTRATOR_LOG` > `RUST_LOG` > `[tracing] default_directives` >
/// [`DEFAULT_LOGGING_DIRECTIVES`]. When an OTLP endpoint is configured, spans
/// are additionally exported to it.
pub fn init_logging(
    bootstrap: Bootstrap,
    settings: Option<&Logging>,
) -> anyhow::Result<Option<SdkTracerProvider>> {
    let tracing_settings = settings.map(Logging::to_tracing_settings);
    opentalk_tracing::init(
        bootstrap,
        tracing_settings.as_ref(),
        TracingParams {
            service_name: "orchestrator",
            service_version: opentalk_tracing::service_version!(),
            default_directives: DEFAULT_LOGGING_DIRECTIVES,
            filter_env_vars: &[LOG_ENV],
        },
    )
}

/// Installs a temporary subscriber used until [`init_logging`] can run, capturing log
/// events emitted while the configuration is still being read. Pass the
/// returned guard to [`init_logging`].
pub fn bootstrap() -> Bootstrap {
    opentalk_tracing::bootstrap(&[LOG_ENV], DEFAULT_LOGGING_DIRECTIVES)
}

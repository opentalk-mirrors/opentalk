// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_tracing::{Bootstrap, SdkTracerProvider, TracingParams, TracingSettings};

/// Environment variable used to configure the log filter, taking precedence over `RUST_LOG`.
const LOG_ENV: &str = "RECORDER_LOG";

/// Default log directives, applied when neither `RUST_LOG` nor the configuration provide any.
const DEFAULT_LOGGING_DIRECTIVES: &str = "warn,opentalk_recorder=info";

/// Initializes the global tracing subscriber.
///
/// The log filter directives are resolved with the following precedence:
/// `RECORDER_LOG` > `RUST_LOG` > `[tracing] default_directives` >
/// [`DEFAULT_LOGGING_DIRECTIVES`]. When an OTLP endpoint is configured, spans
/// are additionally exported to it.
pub fn init(
    bootstrap: Bootstrap,
    settings: Option<&TracingSettings>,
) -> anyhow::Result<Option<SdkTracerProvider>> {
    opentalk_tracing::init(
        bootstrap,
        settings,
        TracingParams {
            service_name: "recorder",
            service_version: opentalk_tracing::service_version!(),
            default_directives: DEFAULT_LOGGING_DIRECTIVES,
            filter_env_vars: &[LOG_ENV],
        },
    )
}

/// Installs a temporary subscriber used until [`init`] can run, capturing log
/// events emitted while the configuration is still being read. Pass the
/// returned guard to [`init`].
pub fn bootstrap() -> Bootstrap {
    opentalk_tracing::bootstrap(&[LOG_ENV], DEFAULT_LOGGING_DIRECTIVES)
}

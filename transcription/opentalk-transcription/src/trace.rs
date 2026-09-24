// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_tracing::{SdkTracerProvider, TracingParams, TracingSettings};

/// Environment variable used to configure the log filter, taking precedence over `RUST_LOG`.
const LOG_ENV: &str = "TRANSCRIPTION_LOG";

/// Default log directives, applied when neither [`LOG_ENV`] nor `RUST_LOG` is set.
const DEFAULT_LOGGING_DIRECTIVES: &str = "warn,opentalk_transcription=info";

/// Initializes the global tracing subscriber.
///
/// The log filter directives are resolved with the following precedence:
/// `TRANSCRIPTION_LOG` > `RUST_LOG` > `[tracing] default_directives` >
/// [`DEFAULT_LOGGING_DIRECTIVES`]. When an OTLP endpoint is configured, spans
/// are additionally exported to it.
pub(crate) fn init(
    settings: Option<&TracingSettings>,
) -> anyhow::Result<Option<SdkTracerProvider>> {
    opentalk_tracing::init(
        settings,
        TracingParams {
            service_name: "transcription",
            service_version: opentalk_tracing::service_version!(),
            default_directives: DEFAULT_LOGGING_DIRECTIVES,
            filter_env_vars: &[LOG_ENV],
        },
    )
}

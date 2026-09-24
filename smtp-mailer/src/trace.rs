// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_tracing::{SdkTracerProvider, TracingParams, TracingSettings};

/// Default log directives, applied when neither `RUST_LOG` nor the configuration provide any.
const DEFAULT_LOGGING_DIRECTIVES: &str = "warn,html5ever=warn,selectors=warn";

/// Initializes the global tracing subscriber.
///
/// The log filter directives are resolved with the following precedence:
/// `RUST_LOG` > `[tracing] default_directives` > [`DEFAULT_LOGGING_DIRECTIVES`].
/// When an OTLP endpoint is configured, spans are additionally exported to it.
pub(crate) fn init(
    settings: Option<&TracingSettings>,
) -> anyhow::Result<Option<SdkTracerProvider>> {
    opentalk_tracing::init(
        settings,
        TracingParams {
            service_name: "smtp-mailer",
            service_version: opentalk_tracing::service_version!(),
            default_directives: DEFAULT_LOGGING_DIRECTIVES,
            filter_env_vars: &[],
        },
    )
}

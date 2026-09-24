// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_common::settings::Tracing;
use opentalk_tracing::{SdkTracerProvider, TracingParams};

/// Environment variable used to configure the log filter, taking precedence over `RUST_LOG`.
const LOG_ENV: &str = "ROOMSERVER_LOG";

/// Default log directives, applied when neither `RUST_LOG` nor the configuration provide any.
const DEFAULT_LOGGING_DIRECTIVES: &str =
    "warn,opentalk_roomserver=info,opentalk_orchestrator_client=info";

/// Initializes the global tracing subscriber.
///
/// The log filter directives are resolved with the following precedence:
/// `ROOMSERVER_LOG` > `RUST_LOG` > `[tracing] default_directives` >
/// [`DEFAULT_LOGGING_DIRECTIVES`]. When an OTLP endpoint is configured, spans
/// are additionally exported to it.
pub fn init(settings: Option<&Tracing>) -> anyhow::Result<Option<SdkTracerProvider>> {
    opentalk_tracing::init(
        settings,
        TracingParams {
            service_name: "roomserver",
            service_version: opentalk_tracing::service_version!(),
            default_directives: DEFAULT_LOGGING_DIRECTIVES,
            filter_env_vars: &[LOG_ENV],
        },
    )
}

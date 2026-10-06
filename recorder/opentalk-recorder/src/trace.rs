// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_tracing::{Bootstrap, SdkTracerProvider, TracingParams, TracingSettings};

const DEFAULT_LOGGING_DIRECTIVES: &str = "warn,opentalk_recorder=info";

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
            filter_env_vars: &[],
        },
    )
}

/// Installs a temporary subscriber used until [`init`] can run, capturing log
/// events emitted while the configuration is still being read. Pass the
/// returned guard to [`init`].
pub fn bootstrap() -> Bootstrap {
    opentalk_tracing::bootstrap(&[], DEFAULT_LOGGING_DIRECTIVES)
}

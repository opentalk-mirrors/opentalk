// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_tracing::{SdkTracerProvider, TracingParams, TracingSettings};

const DEFAULT_LOGGING_DIRECTIVES: &str = "warn,opentalk_recorder=info";

pub fn init(settings: Option<&TracingSettings>) -> anyhow::Result<Option<SdkTracerProvider>> {
    opentalk_tracing::init(
        settings,
        TracingParams {
            service_name: "recorder",
            service_version: opentalk_tracing::service_version!(),
            default_directives: DEFAULT_LOGGING_DIRECTIVES,
            filter_env_vars: &[],
        },
    )
}

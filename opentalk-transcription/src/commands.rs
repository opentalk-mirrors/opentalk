// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::process::ExitCode;

use anyhow::{Context, Result};
use reqwest::Url;
use service_probe_client::is_ready;

use crate::settings::Settings;

/// Queries the readiness state of a running transcription service via its monitoring endpoint.
///
/// Returns [`ExitCode::SUCCESS`] when the service is ready, [`ExitCode::FAILURE`] when it is not
/// ready (or no monitoring endpoint is available), and a non-zero code on query errors.
pub(crate) async fn run_health_command(
    endpoint: Option<Url>,
    settings: &Settings,
) -> Result<ExitCode> {
    let monitoring_endpoint = match endpoint.or(monitoring_endpoint_from_settings(settings)?) {
        Some(endpoint) => endpoint,
        None => {
            log::warn!("Monitoring not configured and no url endpoint parameter given");
            return Ok(ExitCode::FAILURE);
        }
    };

    match is_ready(&monitoring_endpoint).await {
        Ok(true) => {
            log::info!("READY");
            Ok(ExitCode::SUCCESS)
        }
        Ok(false) => {
            log::info!("Not Ready");
            Ok(ExitCode::FAILURE)
        }
        Err(err) => {
            log::error!("Failed to query readiness state: {err}");
            Ok(ExitCode::from(2))
        }
    }
}

/// Builds the monitoring endpoint URL from the configured monitoring settings, if present.
fn monitoring_endpoint_from_settings(settings: &Settings) -> Result<Option<Url>> {
    settings
        .monitoring
        .as_ref()
        .map(|monitoring| {
            format!("http://{}:{}", monitoring.addr, monitoring.port)
                .parse()
                .context("Failed to build monitoring endpoint from configuration")
        })
        .transpose()
}

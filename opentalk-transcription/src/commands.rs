// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::process::exit;

use anyhow::Result;
use reqwest::Url;
use service_probe_client::is_ready;

use crate::settings::Settings;

/// Queries the readiness state of a running transcription service via its monitoring endpoint.
pub(crate) async fn run_health_command(endpoint: Option<Url>, settings: &Settings) -> Result<()> {
    let Some(monitoring_endpoint) = endpoint.or_else(|| {
        settings.monitoring.as_ref().map(|monitoring_settings| {
            format!(
                "http://{}:{}",
                monitoring_settings.addr, monitoring_settings.port
            )
            .parse()
            .expect("valid endpoint can be built from monitoring settings")
        })
    }) else {
        log::warn!("Monitoring not configured and no url endpoint parameter given");
        exit(1);
    };

    match is_ready(&monitoring_endpoint).await {
        Ok(true) => {
            log::info!("READY");
            Ok(())
        }
        Ok(false) => {
            log::info!("Not Ready");
            exit(1)
        }
        Err(err) => {
            log::error!("Err: {err}");
            exit(-1)
        }
    }
}

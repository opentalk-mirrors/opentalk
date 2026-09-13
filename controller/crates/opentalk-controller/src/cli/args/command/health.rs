// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{path::Path, process::exit};

use clap::Parser;
use opentalk_controller_core::load_settings_provider;
use service_probe_client::is_ready;
use url::Url;

use crate::Result;
#[derive(Debug, Clone, Parser)]
pub struct Command {
    /// The monitoring endpoint can be provided optionally
    endpoint: Option<Url>,
}

impl Command {
    pub async fn exec(self, optional_config_path: Option<&Path>) -> Result<()> {
        let settings = load_settings_provider(optional_config_path)?.get();
        let Some(monitoring_endpoint) = self.endpoint.or_else(|| {
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

        return match is_ready(&monitoring_endpoint).await {
            Ok(is_ready) => match is_ready {
                true => {
                    log::info!("READY");
                    Ok(())
                }
                false => {
                    log::info!("Not Ready");
                    exit(1)
                }
            },
            Err(err) => {
                log::error!("Err: {}", err);
                exit(-1)
            }
        };
    }
}

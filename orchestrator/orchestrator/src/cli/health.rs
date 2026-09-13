// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{path::PathBuf, process::exit};

use anyhow::Context;
use service_probe_client::is_ready;
use url::Url;

use crate::settings::Settings;

#[derive(clap::Args, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub struct HealthArgs {
    /// The endpoint to check the health of. If not provided, the endpoint is read from the config
    /// file.
    endpoint: Option<Url>,
}

pub async fn health_check(config: Option<PathBuf>, args: HealthArgs) -> anyhow::Result<()> {
    let url = match args.endpoint {
        Some(url) => url,
        None => {
            let config_path = config.as_deref();
            let settings = Settings::load(config_path)?;

            if let Some(monitoring) = &settings.monitoring {
                let address = monitoring.address;
                let port = monitoring.port;

                Url::parse(&format!("http://{address}:{port}"))?
            } else {
                anyhow::bail!(
                    "No endpoint provided as argument and no monitoring configuration found in settings"
                );
            }
        }
    };

    tracing::debug!("Checking readiness of orchestrator at {url}");

    if is_ready(&url).await.context("Failed to get ready state")? {
        println!("READY");
        Ok(())
    } else {
        println!("NOT READY");
        exit(1)
    }
}

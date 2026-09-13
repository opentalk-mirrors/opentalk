// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{path::Path, process::exit, sync::Arc};

use opentalk_roomserver_common::settings::{Settings, SettingsFile};
use service_probe_client::is_ready;
use url::Url;

#[derive(clap::Args, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub struct Args {
    /// The monitoring endpoint can be provided optionally
    endpoint: Option<Url>,
}

pub(crate) async fn handle_command(
    args: Args,
    config_file_path: Option<&Path>,
) -> anyhow::Result<()> {
    let settings: Arc<Settings> = Arc::new(SettingsFile::load(config_file_path)?.try_into()?);
    let Args { endpoint } = args;

    let endpoint_url = if let Some(endpoint) = endpoint {
        endpoint
    } else if let Some(monitoring_settings) = &settings.monitoring {
        monitoring_settings.url()?
    } else {
        tracing::error!("Monitoring not configured and no url endpoint parameter given");
        exit(1);
    };

    if is_ready(&endpoint_url).await? {
        println!("READY");
        Ok(())
    } else {
        println!("NOT READY");
        exit(1)
    }
}

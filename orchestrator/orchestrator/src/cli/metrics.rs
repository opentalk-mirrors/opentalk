// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::path::PathBuf;

use anyhow::{Context, bail};
use opentalk_service_auth::ApiKey;
use url::Url;

use crate::settings::Settings;

#[derive(clap::Args, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub struct MetricsArgs {
    /// The endpoint to fetch the metrics from. If not provided, the endpoint is read from the
    /// config file.
    endpoint: Option<Url>,
    /// The API key to authorize the request. If not provided, the first API key from the config
    /// file is used.
    api_key: Option<ApiKey>,
}

/// Calls the metrics endpoint of the orchestrator and prints the response body to stdout.
///
/// Uses the provided endpoint and api key, or falls back to the values from the config file if not
/// provided. The config file is read from the provided path or from the default locations if the
/// path is not provided.
pub async fn fetch_metrics(
    config_path: Option<PathBuf>,
    MetricsArgs {
        endpoint: url,
        api_key,
    }: MetricsArgs,
) -> anyhow::Result<()> {
    let (url, api_key) = match (url, api_key) {
        (Some(url), Some(api_key)) => (url, api_key),
        (url, api_key) => {
            let settings = Settings::load(config_path.as_deref())?;

            let url = url.unwrap_or(metrics_url_from_settings(&settings)?);
            let api_key = api_key.unwrap_or(
                settings
                    .http
                    .api_keys
                    .inner()
                    .first()
                    .cloned()
                    .context("Missing api key in config")?,
            );

            (url, api_key)
        }
    };

    let client = reqwest::Client::new();

    let response = client
        .get(url.clone())
        .bearer_auth(api_key.generate_jwt().context("Failed to generate jwt")?)
        .send()
        .await
        .with_context(|| format!("Failed to send request HTTP request to {}", url))?;

    let status = response.status();
    let body = response
        .bytes()
        .await
        .context("Failed to read response body")?;

    if !status.is_success() {
        bail!(
            "received non-success status code {status} with body: {}",
            String::from_utf8_lossy(&body)
        );
    }

    let json: serde_json::Value = serde_json::from_slice(&body)?;
    let body = serde_json::to_string_pretty(&json)?;

    println!("{body}");

    Ok(())
}

fn metrics_url_from_settings(settings: &Settings) -> anyhow::Result<Url> {
    Ok(format!(
        "http://{}:{}/metrics",
        settings.http.address, settings.http.port
    )
    .parse()?)
}

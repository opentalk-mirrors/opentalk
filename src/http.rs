//! HTTP calls made by this library (except for websockets)

use crate::settings::ControllerSettings;
use crate::websocket::Ticket;
use anyhow::{bail, Result};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

/// Error returned by the `start` function when the given digits were incorrect
#[derive(Debug, thiserror::Error)]
#[error("given credentials were invalid")]
pub struct InvalidCredentials;

#[derive(Serialize)]
struct StartRequest<'s> {
    sip_id: &'s str,
    password: &'s str,
}

#[derive(Deserialize)]
struct StartResponse {
    ticket: String,
}

/// Request a signaling ticket with the given DTMF digits `sip_id` and `password`
///
/// The ticket is then used to establish a websocket connection
pub async fn start(settings: &ControllerSettings, sip_id: &str, password: &str) -> Result<Ticket> {
    let client = reqwest::Client::new();

    let uri = if settings.insecure {
        log::warn!("using insecure connection");
        format!("http://{}/v1/rooms/sip/start", settings.domain)
    } else {
        format!("https://{}/v1/rooms/sip/start", settings.domain)
    };

    let response = client
        .post(uri)
        .header("content-type", "application/json")
        .body(serde_json::to_string(&StartRequest { sip_id, password })?)
        .send()
        .await?;

    if response.status() == StatusCode::UNAUTHORIZED {
        bail!(InvalidCredentials);
    }

    let body = response.bytes().await?;

    let response: StartResponse = serde_json::from_slice(&body)?;

    Ok(Ticket(response.ticket))
}

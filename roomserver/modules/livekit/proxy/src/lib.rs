// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use anyhow::{anyhow, bail};
use futures::{SinkExt as _, StreamExt as _};
use opentalk_roomserver_types::livekit_proxy::{
    PreparedSocket,
    websocket::{CloseFrame, LiveKitSocket, LiveKitSocketMessage},
};
use tokio::sync::oneshot;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        Message as TungsteniteMessage,
        client::IntoClientRequest,
        http::{HeaderMap, header::AUTHORIZATION},
        protocol::{CloseFrame as TungsteniteCloseFrame, frame::coding::CloseCode},
    },
};
use url::Url;

/// A wrapper around `oneshot::Sender<()>` that sends on drop,
/// ensuring the shutdown signal is always delivered.
#[derive(Debug)]
pub struct ShutdownSender(Option<oneshot::Sender<()>>);

impl ShutdownSender {
    pub fn new() -> (Self, oneshot::Receiver<()>) {
        let (tx, rx) = oneshot::channel();
        (Self(Some(tx)), rx)
    }
}

impl Drop for ShutdownSender {
    fn drop(&mut self) {
        if let Some(tx) = self.0.take() {
            let _ = tx.send(());
        }
    }
}

/// Connects to the upstream LiveKit server and returns the WebSocket connection.
pub async fn connect_to_livekit(
    mut livekit_rtc_url: Url,
    raw_query: Option<String>,
    mut downstream_headers: HeaderMap,
) -> anyhow::Result<PreparedSocket> {
    livekit_rtc_url.set_query(raw_query.as_deref());
    let mut request = livekit_rtc_url
        .as_str()
        .into_client_request()
        .map_err(|err| anyhow!("failed to build livekit websocket request: {err}"))?;

    // Only pass authorization header to LiveKit, ignore all other header
    if let Some(authorization) = downstream_headers.remove(AUTHORIZATION) {
        request.headers_mut().append(AUTHORIZATION, authorization);
    }

    let (upstream_socket, _) = connect_async(request)
        .await
        .map_err(|err| anyhow!("failed to connect to livekit websocket: {err}"))?;

    Ok(upstream_socket)
}

pub async fn proxy_websocket(
    upstream_socket: PreparedSocket,
    downstream_socket: Box<dyn LiveKitSocket>,
    mut shutdown_rx: oneshot::Receiver<()>,
) -> anyhow::Result<()> {
    let (mut upstream_sink, mut upstream_stream) = upstream_socket.split();
    let (mut downstream_sink, mut downstream_stream) = downstream_socket.split();

    loop {
        tokio::select! {
            _ = &mut shutdown_rx => {
                break;
            }
            item = downstream_stream.next() => {
                let Some(item) = item else {
                    break;
                };

                let item = match item {
                    Ok(item) => item,
                    Err(err) => {
                        tracing::debug!("error reading downstream livekit socket: {err:#}");
                        break;
                    }
                };

                let Some(message) = map_downstream_message(item) else {
                    continue;
                };
                let is_close = matches!(message, TungsteniteMessage::Close(_));

                if upstream_sink.send(message).await.is_err() {
                    break;
                }


                if is_close {
                    break;
                }
            }
            item = upstream_stream.next() => {
                let Some(item) = item else {
                    break;
                };

                let message = match item {
                    Ok(message) => message,
                    Err(err) => {
                        tracing::debug!("error reading upstream livekit socket: {err:#}");
                        break;
                    }
                };

                let Some(message) = map_upstream_message(message) else {
                    continue;
                };

                let is_close = matches!(message, LiveKitSocketMessage::Close(_));
                if downstream_sink.send(message).await.is_err() {
                    break;
                }

                if is_close {
                    break;
                }
            }
        }
    }

    Ok(())
}

pub fn build_livekit_rtc_url(service_url: &Url) -> Result<Url, anyhow::Error> {
    let mut url = service_url.clone();
    if url.scheme() == "https" {
        url.set_scheme("wss")
            .map_err(|_| anyhow!("Invalid scheme"))?;
    } else if url.scheme() == "http" {
        url.set_scheme("ws")
            .map_err(|_| anyhow!("Invalid scheme"))?;
    } else {
        bail!("unsupported scheme");
    }

    // Do not use join as we want to not replace the last segment if a trailing
    // slash is missing.
    url.path_segments_mut()
        .map_err(|_| anyhow!("Invalid URL cannot be base"))?
        .push("rtc");

    Ok(url)
}

fn map_downstream_message(message: LiveKitSocketMessage) -> Option<TungsteniteMessage> {
    match message {
        LiveKitSocketMessage::Text(text) => Some(TungsteniteMessage::Text(text.into())),
        LiveKitSocketMessage::Binary(binary) => Some(TungsteniteMessage::Binary(binary)),
        LiveKitSocketMessage::Ping(bytes) => Some(TungsteniteMessage::Ping(bytes)),
        LiveKitSocketMessage::Pong(_) => None,
        LiveKitSocketMessage::Close(close_frame) => {
            Some(TungsteniteMessage::Close(close_frame.map(|frame| {
                TungsteniteCloseFrame {
                    code: CloseCode::from(frame.code),
                    reason: frame.reason.into(),
                }
            })))
        }
    }
}

fn map_upstream_message(message: TungsteniteMessage) -> Option<LiveKitSocketMessage> {
    match message {
        TungsteniteMessage::Text(text) => Some(LiveKitSocketMessage::Text(text.to_string())),
        TungsteniteMessage::Binary(binary) => Some(LiveKitSocketMessage::Binary(binary)),
        TungsteniteMessage::Ping(bytes) => Some(LiveKitSocketMessage::Ping(bytes)),
        TungsteniteMessage::Pong(_) => None,
        TungsteniteMessage::Close(close_frame) => {
            Some(LiveKitSocketMessage::Close(close_frame.map(|frame| {
                CloseFrame {
                    code: frame.code.into(),
                    reason: frame.reason.to_string(),
                }
            })))
        }
        TungsteniteMessage::Frame(_) => None,
    }
}

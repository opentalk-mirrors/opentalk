// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use anyhow::{Context as _, anyhow};
use futures::{SinkExt as _, StreamExt};
use opentalk_types_common::roomserver::Token;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tungstenite::{
    ClientRequestBuilder, Message, Utf8Bytes,
    protocol::{CloseFrame, frame::coding::CloseCode},
};
use url::Url;

use crate::api::{command::SignalingCommand, event::SignalingEvent};

#[derive(Debug, Error)]
#[error("Signaling connection error")]
pub struct SignalingError {
    #[from]
    source: anyhow::Error,
}

#[derive(Debug)]
pub struct SignalingConnection {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl SignalingConnection {
    pub async fn connect(roomserver_url: Url, token: Token) -> Result<Self, SignalingError> {
        let uri = build_signaling_socket_url(&roomserver_url, &token)?;

        tracing::debug!("connect signaling to url: {uri}");
        let builder = ClientRequestBuilder::new(uri);
        let (socket, _response) = connect_async(builder)
            .await
            .context("Failed to open signaling connection")?;

        Ok(Self { socket })
    }

    pub async fn close(&mut self) -> Result<(), SignalingError> {
        self.socket
            .close(Some(CloseFrame {
                code: CloseCode::Away,
                reason: Utf8Bytes::from_static(""),
            }))
            .await
            .context("Failed to close signaling socket")?;

        Ok(())
    }

    pub async fn send_raw_message(&mut self, message: &str) -> Result<(), SignalingError> {
        tracing::trace!("send text message: {message:?}");
        self.socket
            .send(Message::Text(message.into()))
            .await
            .context("Failed to send message")?;

        Ok(())
    }

    pub async fn receive_raw_message(&mut self) -> Result<Option<String>, SignalingError> {
        while let Some(msg) = self.socket.next().await {
            let msg = msg.context("receive error")?;

            tracing::trace!("received message: {msg:?}");

            return match msg {
                Message::Text(utf8_bytes) => Ok(Some(utf8_bytes.to_string())),

                // don't log the full message, just the type
                Message::Binary(_) => Err(anyhow!("Expected text message, got: Binary").into()),
                Message::Close(_) => Ok(None),
                Message::Frame(_) => Err(anyhow!("Expected text message, got: Frame").into()),
                // ignore ping/pong messages, these shouldn't be printed and aren't returning any
                // data
                Message::Ping(_) | Message::Pong(_) => continue,
            };
        }

        Ok(None)
    }

    pub async fn receive(&mut self) -> Result<Option<SignalingEvent>, SignalingError> {
        match self.receive_raw_message().await {
            Ok(Some(message)) => Ok(Some(
                serde_json::from_str::<SignalingEvent>(&message).context("Failed to serialize")?,
            )),

            // map the option type from String to SignalingEvent.
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn send(&mut self, command: &SignalingCommand) -> Result<(), SignalingError> {
        let message = serde_json::to_string(&command).context("Failed to serialize command")?;
        self.send_raw_message(&message).await
    }
}

fn build_signaling_socket_url(roomserver_url: &Url, token: &Token) -> anyhow::Result<http::Uri> {
    let mut url = roomserver_url
        .join("v1/signaling/")
        .context("Internal error, failed to append `signaling` path, invalid url")?
        .join(&token.to_string())
        .context("Internal error, failed to append signaling token to path, invalid url")?;
    match url.scheme() {
        "https" => url
            .set_scheme("wss")
            .map_err(|()| anyhow!("Failed to set scheme"))?,
        _ => url
            .set_scheme("ws")
            .map_err(|()| anyhow!("Failed to set scheme"))?,
    }
    let uri = http::Uri::try_from(url.to_string())
        .context("Internal error, failed to convert url to uri")?;
    Ok(uri)
}

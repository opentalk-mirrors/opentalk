// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use core::fmt::Debug;

use anyhow::{Context, Result, bail};
use futures_util::{SinkExt, StreamExt};
use opentalk_service_auth::EncodingError;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream,
    tungstenite::{
        self, Message as TtMessage, Utf8Bytes,
        client::IntoClientRequest,
        http::{
            HeaderValue, Request,
            header::{AUTHORIZATION, InvalidHeaderValue, SEC_WEBSOCKET_PROTOCOL},
        },
        protocol::{CloseFrame, frame::coding::CloseCode},
    },
};

use crate::{
    client::ClientError,
    config::{OrchestratorConfig, UrlError},
};

const ORCHESTRATOR_PROTOCOL_HEADER: HeaderValue =
    HeaderValue::from_static("opentalk-orchestrator-json-v1.0");

type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Websocket wrapper for orchestrator signaling
#[derive(Debug)]
pub(crate) struct SignalingSocket {
    /// The websocket connection to the orchestrator
    ///
    /// Is [None] when the connection has been closed
    websocket: Option<WebSocket>,
}

impl SignalingSocket {
    /// Establish a websocket connection with the configured orchestrator
    pub(crate) async fn connect(config: &OrchestratorConfig) -> Result<Self, ClientError> {
        let request = build_websocket_request(config)?;

        let (websocket, _) = tokio_tungstenite::connect_async(request)
            .await
            .context("Failed to establish connection with the orchestrator")?;

        Ok(Self {
            websocket: Some(websocket),
        })
    }

    pub(crate) async fn recv<T: for<'a> Deserialize<'a>>(&mut self) -> Result<T> {
        let payload = self.recv_websocket_message().await?;

        serde_json::from_str(&payload).context("Failed to parse websocket message")
    }

    /// Receive any text data from the websocket
    async fn recv_websocket_message(&mut self) -> Result<Utf8Bytes> {
        loop {
            match self
                .socket()?
                .next()
                .await
                .transpose()
                .context("Failed to receive websocket message")?
            {
                Some(TtMessage::Text(data)) => {
                    return Ok(data);
                }
                Some(TtMessage::Binary(bytes)) => {
                    return Utf8Bytes::try_from(bytes)
                        .context("Received non-UTF8 bytes from orchestrator");
                }
                Some(TtMessage::Close(close_frame)) => {
                    log::debug!(
                        "received close message from orchestrator: {}",
                        close_frame
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or("<no close frame>".into())
                    );

                    // close handshake is handled by tungstenite (see https://github.com/snapview/tokio-tungstenite/issues/207)
                    bail!("Disconnected form Orchestrator, socket was closed by the server");
                }
                Some(_) => {}
                None => bail!(
                    "Disconnected form Orchestrator, socket was unexpectedly closed by the server"
                ),
            }
        }
    }

    pub(crate) async fn send<P>(&mut self, payload: P) -> Result<()>
    where
        P: Serialize,
    {
        self.socket()?
            .send(TtMessage::Text(serde_json::to_string(&payload)?.into()))
            .await?;

        Ok(())
    }

    /// Close the underlying websocket connection
    ///
    /// Any further calls to send or recv will result in [SignalingError::AlreadyClosed]
    pub(crate) async fn close(&mut self) {
        let Some(mut websocket) = self.websocket.take() else {
            //already closed
            return;
        };

        if let Err(e) = websocket
            .close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: Utf8Bytes::default(),
            }))
            .await
        {
            log::debug!("Failed to send close frame: {e}");
        }
    }

    fn socket(&mut self) -> Result<&mut WebSocket> {
        self.websocket
            .as_mut()
            .context("Unexpected internal error, attempted to operate on closed websocket")
    }
}

impl Drop for SignalingSocket {
    fn drop(&mut self) {
        if let Some(mut websocket) = self.websocket.take() {
            tokio::spawn(async move {
                let Err(err) = websocket.close(None).await else {
                    return;
                };

                match err {
                    tungstenite::Error::ConnectionClosed | tungstenite::Error::AlreadyClosed => (),
                    error => {
                        log::debug!("Failed to properly close websocket connection: {error}");
                    }
                }
            });
        }
    }
}

/// Errors that can occur when building the initial websocket request
#[derive(Debug, thiserror::Error)]
pub enum BuildWsRequestError {
    #[error("Failed to parse orchestrator url")]
    Url(#[from] UrlError),

    #[error(transparent)]
    Encoding(#[from] EncodingError),

    #[error("Failed to build websocket client request: {0}")]
    ClientRequest(#[from] tungstenite::Error),

    #[error("Failed to create authorization header value: {0}")]
    InvalidAuthHeader(#[from] InvalidHeaderValue),
}

fn build_websocket_request(
    config: &OrchestratorConfig,
) -> Result<Request<()>, BuildWsRequestError> {
    let mut websocket_request = config
        .url
        .register_endpoint()?
        .to_string()
        .into_client_request()?;

    websocket_request.headers_mut().insert(
        AUTHORIZATION,
        format!("Bearer {}", config.api_key.generate_jwt()?).try_into()?,
    );

    websocket_request
        .headers_mut()
        .insert(SEC_WEBSOCKET_PROTOCOL, ORCHESTRATOR_PROTOCOL_HEADER);

    Ok(websocket_request)
}

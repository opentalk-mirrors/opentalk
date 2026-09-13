// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Context;
use axum::extract::ws::WebSocket;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub async fn proxy_signaling_task(
    client: WebSocket,
    service: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
) {
    if let Err(e) = forward_signaling_messages(client, service).await {
        tracing::debug!("Proxy task exited with error: {e:?}");
    }
}

async fn forward_signaling_messages(
    mut client: WebSocket,
    mut service: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
) -> anyhow::Result<()> {
    loop {
        tokio::select! {
            msg = client.recv() => {
                let Some(msg) = msg else {
                    tracing::debug!("Client closed connection, ending proxy task");
                    return Ok(());
                };
                let msg = msg.context("Error on client connection")?;

                let Some(msg) = translation::axum_to_tungstenite(msg) else {
                    continue;
                };

                service.send(msg).await?;
            }
            msg = service.next() => {
                let Some(msg) = msg else {
                    tracing::debug!("Roomserver closed connection, ending proxy task");
                    return Ok(());
                };
                let msg = msg.context("Error on roomserver connection")?;

                let Some(msg) = translation::tungstenite_to_axum(msg) else {
                    continue;
                };

                client.send(msg).await?;
            }
        }
    }
}

pub mod translation {
    use axum::extract::ws::{
        CloseFrame as AxumCloseFrame, Message as AxumMessage, Utf8Bytes as AxumUtf8Bytes,
    };
    use bytes::Bytes;
    use tokio_tungstenite::tungstenite::{
        Message as TungsteniteMessage,
        protocol::frame::{
            CloseFrame as TungsteniteCloseFrame, Utf8Bytes as TungsteniteUtf8Bytes,
            coding::CloseCode as TungsteniteCloseCode,
        },
    };

    pub fn axum_to_tungstenite(msg: AxumMessage) -> Option<TungsteniteMessage> {
        match msg {
            AxumMessage::Text(text) => Some(TungsteniteMessage::Text(
                // Safety: UTF8 invariant is uphold because the source type is guaranteed to be
                // UTF8
                unsafe { TungsteniteUtf8Bytes::from_bytes_unchecked(Bytes::from(text)) },
            )),
            AxumMessage::Binary(bin) => Some(TungsteniteMessage::Binary(bin)),
            AxumMessage::Close(close_frame) => {
                Some(TungsteniteMessage::Close(close_frame.map(|axum_close| {
                    TungsteniteCloseFrame {
                        code: TungsteniteCloseCode::from(axum_close.code),
                        reason: TungsteniteUtf8Bytes::from(axum_close.reason.as_str()),
                    }
                })))
            }
            AxumMessage::Ping(ping) => Some(TungsteniteMessage::Ping(ping)),
            // tungstenite automatically responds to pings, forwarding pongs would result in two
            // pongs on the receiver
            AxumMessage::Pong(_) => None,
        }
    }

    pub fn tungstenite_to_axum(msg: TungsteniteMessage) -> Option<AxumMessage> {
        match msg {
            TungsteniteMessage::Text(text) => {
                Some(AxumMessage::Text(
                    // Safety: UTF8 invariant is uphold because the source type is guaranteed to be
                    // UTF8
                    AxumUtf8Bytes::try_from(Bytes::from(text)).expect("valid UTF8 bytes"),
                ))
            }
            TungsteniteMessage::Binary(bin) => Some(AxumMessage::Binary(bin)),
            TungsteniteMessage::Close(close_frame) => {
                Some(AxumMessage::Close(close_frame.map(|tungstenite_close| {
                    AxumCloseFrame {
                        code: axum::extract::ws::CloseCode::from(tungstenite_close.code),
                        reason: AxumUtf8Bytes::from(tungstenite_close.reason.as_str()),
                    }
                })))
            }
            TungsteniteMessage::Ping(ping) => Some(AxumMessage::Ping(ping)),
            // tungstenite automatically responds to pings, forwarding pongs would result in two
            // pongs on the receiver
            TungsteniteMessage::Pong(_) => None,
            TungsteniteMessage::Frame(_) => {
                tracing::warn!(
                    "Received raw tungstenite frame in websocket proxy, this was unexpected"
                );
                None
            }
        }
    }
}

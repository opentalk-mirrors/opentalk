// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::{Duration, Instant};

use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_http::ws::{CloseCode, CloseReason, Item, ProtocolError};
use actix_web_actors::ws::{Message, WebsocketContext};
use bytes::BytesMut;
use opentalk_asset_storage::ObjectStorageError;
use tokio::sync::mpsc::UnboundedSender;

pub const MAXIMUM_WEBSOCKET_BUFFER_SIZE: usize = 100_000_000;

/// Define HTTP Websocket actor
///
/// This actor will relay all text and binary received websocket messages to the given unbounded sender
/// It is up to the receiver of the channel to extract the underlying message.
///
/// Handling timeouts is also done in this actor.
pub struct UploadWebSocketActor {
    /// Sender to signaling runner
    sender: UnboundedSender<Result<bytes::Bytes, opentalk_asset_storage::ObjectStorageError>>,

    /// Timestamp of last pong received
    last_pong: Instant,

    /// State for receiving fragmented messages
    continuation: Option<Continuation>,
}

struct Continuation {
    buffer: BytesMut,
}

impl UploadWebSocketActor {
    pub fn new(sender: UnboundedSender<Result<bytes::Bytes, ObjectStorageError>>) -> Self {
        Self {
            sender,
            last_pong: Instant::now(),
            continuation: None,
        }
    }

    fn forward_to_runner(
        &mut self,
        ctx: &mut WebsocketContext<Self>,
        bytes: Result<bytes::Bytes, ObjectStorageError>,
    ) {
        if self.sender.send(bytes).is_err() {
            ctx.close(Some(CloseReason {
                code: CloseCode::Abnormal,
                description: Some("runner disconnected".to_owned()),
            }));
        }
    }

    /// Handle continuation packages by saving them in a separate buffer
    fn handle_continuation(&mut self, ctx: &mut WebsocketContext<Self>, item: Item) {
        match item {
            Item::FirstText(bytes) | Item::FirstBinary(bytes) => {
                if self.continuation.is_some() {
                    tracing::warn!("Got continuation while processing one");
                }

                self.continuation = Some(Continuation {
                    buffer: BytesMut::from(&bytes[..]),
                });
            }
            Item::Continue(bytes) => {
                if let Some(continuation) = &mut self.continuation {
                    continuation.buffer.extend_from_slice(&bytes);

                    if continuation.buffer.len() >= MAXIMUM_WEBSOCKET_BUFFER_SIZE {
                        tracing::error!(
                            "Fragmented above the maxium websocket buffer size, stopping actor"
                        );
                        ctx.stop();
                    }
                } else {
                    tracing::warn!("Got continuation continue message without a continuation set");
                }
            }
            Item::Last(bytes) => {
                if let Some(mut continuation) = self.continuation.take() {
                    continuation.buffer.extend_from_slice(&bytes);
                    self.forward_to_runner(ctx, Ok(continuation.buffer.freeze()));
                } else {
                    tracing::warn!("Got continuation last message without a continuation set");
                }
            }
        }
    }
}

impl Actor for UploadWebSocketActor {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Start an interval for connection checks via ping-pong
        ctx.run_interval(Duration::from_secs(15), |this, ctx| {
            if Instant::now().duration_since(this.last_pong) > Duration::from_secs(20) {
                // no response to ping, exit
                ctx.stop();
            } else {
                ctx.ping(b"heartbeat");
            }
        });
    }
}

/// Handle incoming websocket messages
impl StreamHandler<Result<Message, ProtocolError>> for UploadWebSocketActor {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(Message::Ping(msg)) => ctx.pong(&msg),
            Ok(Message::Pong(msg)) => {
                if msg == b"heartbeat"[..] {
                    self.last_pong = Instant::now();
                }
            }
            Ok(Message::Text(text)) => {
                let bytes = text.into_bytes();
                self.forward_to_runner(ctx, Ok(bytes));
            }
            Ok(Message::Binary(bytes)) => {
                self.forward_to_runner(ctx, Ok(bytes));
            }
            Ok(Message::Continuation(item)) => self.handle_continuation(ctx, item),
            Ok(Message::Close(_)) => {
                ctx.close(Some(CloseReason {
                    code: CloseCode::Normal,
                    description: None,
                }));
            }
            Ok(Message::Nop) => {}
            Err(e) => {
                tracing::debug!("Protocol error in websocket - exiting, {e}");

                ctx.stop();
            }
        }
    }
}

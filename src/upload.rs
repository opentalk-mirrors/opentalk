use std::{
    pin::pin,
    time::{Duration, Instant},
};

use futures::{SinkExt, Stream, StreamExt};
use openidconnect::http::Response;
use reqwest::header::HeaderValue;
use tt::{
    WebSocketStream,
    tungstenite::{Message, client::IntoClientRequest},
};

use crate::{OpenTalkClient, http::WebsocketError};

#[derive(Debug, thiserror::Error)]
pub enum UploadingError {
    #[error("Websocket connection failed: {0}")]
    WebsocketConnectionFailed(#[from] WebsocketError),
    #[error("Heartbeat response timed out")]
    HeartbeatTimeout,
    #[error("Stream failed with '{0}'")]
    StreamError(#[from] tt::tungstenite::Error),
    #[error("Sending through channel failed")]
    ChannelError,
    #[error("Authentication failed")]
    AuthenticationFailed,
}

const CHUNK_LIMIT: u32 = 950;

impl OpenTalkClient {
    pub(crate) async fn websocket_connect(
        &self,
        uri: String,
        token: openidconnect::AccessToken,
    ) -> Result<
        (
            WebSocketStream<tt::MaybeTlsStream<tokio::net::TcpStream>>,
            Response<Option<Vec<u8>>>,
        ),
        WebsocketError,
    > {
        let mut request = uri
            .into_client_request()
            .map_err(|_| WebsocketError::UriConversionFailed)?;

        request.headers_mut().insert(
            reqwest::header::AUTHORIZATION,
            HeaderValue::from_str(format!("Bearer {}", token.secret()).as_str())
                .map_err(|_| WebsocketError::InvalidHeader)?,
        );

        tt::connect_async(request)
            .await
            .map_err(WebsocketError::ConnectionFailed)
    }
    /// Opens a websocket connection and uploads all the transfered bytes through the `receiver` object
    ///
    /// # Remarks
    ///
    /// `on_chunk_limit_reached` is called once, when the maximum amount of chunks is about to be reached.
    /// While this does not stop the upload, the stream must be properly flushed and closed as soon as possible.
    ///
    /// Each chunk in the `receiver` must contain a `u32` for the first 4 bytes in `Big Endian`.
    ///
    /// Every chunk must be at least 5 MiB (expect for the last one) and at most 5 GiB.
    pub async fn upload_render<E>(
        &self,
        uri: String,
        chunk_stream: impl Stream<Item = Result<Vec<u8>, E>>,
        on_chunk_limit_reached: impl FnOnce(),
    ) -> Result<(), UploadingError> {
        let mut on_chunk_limit_reached = Some(on_chunk_limit_reached);
        let mut receiver = pin!(chunk_stream);
        let token = self
            .http
            .authentication
            .access_token()
            .await
            .map_err(|_| UploadingError::AuthenticationFailed)?;

        log::debug!("connect websocket to {uri}");
        let (ws_stream, _response) = self.websocket_connect(uri, token).await?;

        let (mut tx, mut rx) = ws_stream.split();

        let mut last_pong = Instant::now();

        let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(15));
        loop {
            tokio::select! {
                Some(message) = rx.next() => {
                    log::trace!("received message {message:?}");
                    match message {
                        Ok(Message::Ping(data)) => tx.send(Message::Pong(data)).await?,
                        Ok(Message::Pong(msg)) => {
                            if msg == b"heartbeat"[..] {
                                last_pong = Instant::now();
                            }
                        }
                        Ok(_) => {}
                        Err(err) => {
                            log::error!("Received websocket error for upload stream: {err:?}");
                            break;
                        }
                    }
                }
                _ = heartbeat_interval.tick() => {
                    if Instant::now().duration_since(last_pong) > Duration::from_secs(20) {
                        tx.close().await?;
                        return Err(UploadingError::HeartbeatTimeout);
                    }

                    tx.send(tt::tungstenite::Message::Ping("heartbeat".into()))
                        .await?;
                }
                result = receiver.next() => {
                    let Some(error_or_bytes) = result else {
                        break;
                    };

                    let Ok(bytes) = error_or_bytes else {
                        return Err(UploadingError::ChannelError);
                    };

                    let part_num = u32::from_be_bytes(bytes[..4].try_into().unwrap_or_default());
                    tx.send(tt::tungstenite::Message::Binary(bytes.into()))
                        .await?;

                    // Limit the chunk count to the fixed maximum chunk amount
                    // by S3, it's defined to be 1000, but we need *some buffer* since we cannot
                    // perfectly control it to stop at exactly the upper limit
                    // so currently, the CHUNK_LIMIT is set to 950 to have some wiggle-room.
                    if part_num > CHUNK_LIMIT {
                        if let Some(on_chunk_limit_reached) = on_chunk_limit_reached.take() {
                            on_chunk_limit_reached();
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

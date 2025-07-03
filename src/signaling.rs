// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use core::fmt::Debug;
use std::collections::VecDeque;

use anyhow::{Context, Result, bail};
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use reqwest::header::SEC_WEBSOCKET_PROTOCOL;
use tokio::net::TcpStream;
use tt::{
    MaybeTlsStream, WebSocketStream,
    tungstenite::{Message as TtMessage, Utf8Bytes, client::IntoClientRequest},
};
use types_common::users::DisplayName;
use types_control::command::{ControlCommand, Join};

use crate::{config::ControllerConfig, http::HttpClient};

#[derive(Debug)]
pub(crate) struct Signaling {
    websocket: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    pending_pings: VecDeque<Bytes>,
}

impl Signaling {
    pub(crate) async fn connect(
        client: &HttpClient,
        settings: &ControllerConfig,
        display_name: DisplayName,
        room_id: &str,
        breakout_id: Option<&str>,
    ) -> Result<Self> {
        let ticket = client.start(settings, room_id, breakout_id).await?;

        let mut websocket_request = settings.websocket_url().into_client_request()?;
        websocket_request.headers_mut().insert(
            SEC_WEBSOCKET_PROTOCOL,
            format!("opentalk-signaling-json-v1.0,ticket#{ticket}").try_into()?,
        );

        let (websocket, _) = tt::connect_async(websocket_request)
            .await
            .context("failed create websocket connection")?;

        let mut this = Self {
            websocket: Some(websocket),
            pending_pings: VecDeque::new(),
        };

        this.send(ControlCommand::Join(Join {
            display_name: Some(display_name),
        }))
        .await?;

        Ok(this)
    }

    /// Run the signaling task until terminated via the shutdown signal
    pub async fn recv(&mut self) -> Result<incoming::Event> {
        while let Some(data) = self.pending_pings.front() {
            self.websocket
                .as_mut()
                .unwrap()
                .send(TtMessage::Pong(data.clone()))
                .await?;
            self.pending_pings.pop_front();
        }

        loop {
            let payload = self.recv_websocket_message().await?;

            if let Ok(payload) = serde_json::from_str(&payload) {
                return Ok(payload);
            }
        }
    }

    // Receive any text data from the websocket and handle other events silently
    async fn recv_websocket_message(&mut self) -> Result<Utf8Bytes> {
        loop {
            match self
                .websocket
                .as_mut()
                .unwrap()
                .next()
                .await
                .transpose()
                .context("Failed to receive data from websocket")?
            {
                Some(TtMessage::Ping(data)) => {
                    self.pending_pings.push_back(data);
                }
                Some(TtMessage::Text(data)) => {
                    return Ok(data);
                }
                Some(TtMessage::Close(c)) => {
                    log::debug!("Received close frame: {c:?}");
                    bail!("received close frame");
                }
                Some(_) => {}
                None => bail!("websocket connection closed"),
            }
        }
    }

    pub async fn send(&mut self, payload: impl Into<outgoing::Command>) -> Result<()> {
        self.websocket
            .as_mut()
            .unwrap()
            .send(TtMessage::Text(
                serde_json::to_string(&payload.into())?.into(),
            ))
            .await?;

        Ok(())
    }
}

impl Drop for Signaling {
    fn drop(&mut self) {
        if let Some(mut websocket) = self.websocket.take() {
            tokio::spawn(async move {
                websocket.close(None).await.unwrap();
            });
        }
    }
}

pub(crate) mod incoming {
    use serde::{Deserialize, Serialize};
    use types_control::event::ControlEvent;
    use types_moderation::event::ModerationEvent;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "namespace", content = "payload", rename_all = "snake_case")]
    #[allow(clippy::large_enum_variant)]
    pub(crate) enum Event {
        Control(ControlEvent),
        Moderation(ModerationEvent),
    }
}

pub(crate) mod outgoing {
    use serde::{Deserialize, Serialize};
    use types_control::command::ControlCommand;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "namespace", content = "payload", rename_all = "snake_case")]
    pub(crate) enum Command {
        #[allow(unused)]
        Control(ControlCommand),
    }

    impl From<ControlCommand> for Command {
        fn from(command: ControlCommand) -> Self {
            Self::Control(command)
        }
    }
}

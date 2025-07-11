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
    tungstenite::{
        Message as TtMessage, Utf8Bytes,
        client::IntoClientRequest,
        protocol::{CloseFrame, frame::coding::CloseCode},
    },
};
use types::{
    common::{auth::TicketToken, users::DisplayName},
    signaling::control::command::{ControlCommand, Join},
};

use crate::config::ControllerConfig;

#[derive(Debug)]
pub(crate) struct Signaling {
    websocket: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    pending_pings: VecDeque<Bytes>,
}

impl Signaling {
    pub(crate) async fn connect(
        ticket: TicketToken,
        config: &ControllerConfig,
        display_name: DisplayName,
    ) -> Result<Self> {
        let mut websocket_request = config.websocket_url().into_client_request()?;
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

    pub(crate) async fn recv(&mut self) -> Result<Option<incoming::Event>> {
        while let Some(data) = self.pending_pings.front() {
            self.websocket
                .as_mut()
                .unwrap()
                .send(TtMessage::Pong(data.clone()))
                .await?;
            self.pending_pings.pop_front();
        }

        loop {
            let Some(payload) = self.recv_websocket_message().await? else {
                return Ok(None);
            };

            match serde_json::from_str(&payload) {
                Ok(payload) => return Ok(payload),
                Err(e) => log::debug!("Failed to parse event: {e:?}"),
            }
        }
    }

    // Receive any text data from the websocket and handle other events silently
    async fn recv_websocket_message(&mut self) -> Result<Option<Utf8Bytes>> {
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
                    return Ok(Some(data));
                }
                Some(TtMessage::Close(c)) => {
                    log::debug!("Received close frame: {c:?}");
                    return Ok(None);
                }
                Some(_) => {}
                None => bail!("websocket connection closed"),
            }
        }
    }

    pub(crate) async fn send(&mut self, payload: impl Into<outgoing::Command>) -> Result<()> {
        self.websocket
            .as_mut()
            .unwrap()
            .send(TtMessage::Text(
                serde_json::to_string(&payload.into())?.into(),
            ))
            .await?;

        Ok(())
    }

    pub(crate) async fn close(&mut self) -> Result<()> {
        let mut websocket = self.websocket.take().unwrap();

        websocket
            .close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: Utf8Bytes::default(),
            }))
            .await
            .context("Failed to send websocket close frame")
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
    use types::signaling::{
        control::event::ControlEvent, livekit::event::LiveKitEvent,
        moderation::event::ModerationEvent, recording::event::RecordingEvent,
    };

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "namespace", content = "payload", rename_all = "snake_case")]
    #[allow(clippy::large_enum_variant)]
    pub(crate) enum Event {
        Control(ControlEvent),
        Moderation(ModerationEvent),
        LiveKit(LiveKitEvent),
        Recording(RecordingEvent),
    }
}

pub(crate) mod outgoing {
    use serde::{Deserialize, Serialize};
    use types::signaling::{
        chat::command::ChatCommand, control::command::ControlCommand,
        recording::command::RecordingCommand,
    };

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "namespace", content = "payload", rename_all = "snake_case")]
    pub enum Command {
        Chat(ChatCommand),
        Control(ControlCommand),
        Recording(RecordingCommand),
    }

    impl From<ChatCommand> for Command {
        fn from(command: ChatCommand) -> Self {
            Self::Chat(command)
        }
    }

    impl From<ControlCommand> for Command {
        fn from(command: ControlCommand) -> Self {
            Self::Control(command)
        }
    }

    impl From<RecordingCommand> for Command {
        fn from(command: RecordingCommand) -> Self {
            Self::Recording(command)
        }
    }
}

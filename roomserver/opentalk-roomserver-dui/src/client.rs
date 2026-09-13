// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use chrono::Local;
use opentalk_roomserver_client::{
    Client,
    api::signaling::{SignalingConnection, SignalingError},
};
use opentalk_roomserver_types::{
    api::RoomServerAccess, client_parameters::ClientParameters, room_parameters::RoomParameters,
};
use opentalk_service_auth::ApiKey;
use opentalk_types_common::{rooms::RoomId, roomserver::Token};
use tokio::{
    runtime::Runtime,
    sync::{mpsc, watch},
};
use url::Url;

/// A fatal error occurred and the runner should exit.
#[derive(Debug)]
struct FatalError;

pub type RunnerResponse<T> = anyhow::Result<T>;

#[derive(Debug)]
pub enum RunnerCommand {
    RoomServerAccess {
        response_tx: tokio::sync::oneshot::Sender<RunnerResponse<()>>,
        url: Url,
        api_key: ApiKey,
    },

    QueryRoom {
        response_tx: tokio::sync::oneshot::Sender<RunnerResponse<bool>>,
        room_id: RoomId,
    },

    RequestToken {
        response_tx: tokio::sync::oneshot::Sender<RunnerResponse<RoomServerAccess>>,
        room_id: RoomId,
        client_parameters: ClientParameters,
        room_parameters: Box<Option<RoomParameters>>,
    },

    ConnectSignaling {
        response_tx: tokio::sync::oneshot::Sender<RunnerResponse<()>>,
        token: Token,
        url: Url,
    },

    SuspendReceive,

    ResumeReceive,

    Close,

    Send {
        message: String,
    },
}

#[derive(Debug)]
pub enum RunnerEventType {
    Disconnected,
    Connected,
    Received { message: String },
    ReceiveError { error: SignalingError },
    SendSuccess { message: String },
    SendError { error: String },
}

#[derive(Debug)]
pub struct RunnerEvent {
    pub event_type: RunnerEventType,
    pub timestamp: chrono::DateTime<Local>,
}

impl From<RunnerEventType> for RunnerEvent {
    fn from(value: RunnerEventType) -> Self {
        RunnerEvent {
            event_type: value,
            timestamp: Local::now(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SignalingState {
    Connected,
    Disconnect,
}

impl SignalingState {
    /// Returns `true` if the signaling state is [`Connected`].
    ///
    /// [`Connected`]: SignalingState::Connected
    #[must_use]
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }
}

#[derive(Debug)]
pub struct RoomServerRunner {
    egui_ctx: egui::Context,
    client: Client,

    connection: Option<SignalingConnection>,
    receive_suspended: bool,

    event_tx: mpsc::UnboundedSender<RunnerEvent>,
    command_rx: mpsc::UnboundedReceiver<RunnerCommand>,
    signaling_state_tx: watch::Sender<SignalingState>,
}

impl RoomServerRunner {
    pub fn spawn(
        runtime: &Runtime,
        egui_ctx: egui::Context,
        roomserver_url: Url,
        api_key: ApiKey,
    ) -> anyhow::Result<(
        mpsc::UnboundedReceiver<RunnerEvent>,
        mpsc::UnboundedSender<RunnerCommand>,
        watch::Receiver<SignalingState>,
    )> {
        let client = Client::new(roomserver_url, api_key);

        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (signaling_state_tx, signaling_state_rx) = watch::channel(SignalingState::Disconnect);
        let this = Self {
            client,
            connection: None,
            event_tx,
            command_rx,
            signaling_state_tx,
            egui_ctx,
            receive_suspended: false,
        };

        runtime.spawn(this.run());
        Ok((event_rx, command_tx, signaling_state_rx))
    }

    async fn run(mut self) -> Result<(), FatalError> {
        log::info!("Running roomserver runner");
        loop {
            tokio::select! {
                command = self.command_rx.recv() => {
                    if let Some(command) = command {
                        self.process_command(command).await?;
                    } else {
                        log::info!("Command channel closed, exiting...");
                        return Ok(())
                    }
                }
                message = Self::next_received(self.connection.as_mut(), self.receive_suspended) => {
                    self.process_signaling_message(message).await?;
                }
            }
            // request a repaint of the UI each time something happened.
            log::trace!("request repaint: RoomServerRunner event");
            self.egui_ctx.request_repaint();
        }
    }

    /// Wrapper to receive from an optional connection.
    ///
    /// If the connection is none (we are not connected with a signaling websocket) this call will
    /// be stuck in pending.
    async fn next_received(
        conn: Option<&mut SignalingConnection>,
        suspended: bool,
    ) -> Result<Option<String>, SignalingError> {
        match (suspended, conn) {
            (false, Some(conn)) => conn.receive_raw_message().await,
            (_, None) | (true, _) => std::future::pending().await,
        }
    }

    async fn process_signaling_message(
        &mut self,
        message: Result<Option<String>, SignalingError>,
    ) -> Result<(), FatalError> {
        match message {
            Ok(Some(message)) => {
                self.event_tx
                    .send(RunnerEventType::Received { message }.into())
                    .map_err(|_| FatalError)?;
            }
            Ok(None) => {
                self.disconnect().await?;
                self.event_tx
                    .send(RunnerEventType::Disconnected.into())
                    .map_err(|_| FatalError)?;
            }
            Err(error) => {
                self.disconnect().await?;
                self.event_tx
                    .send(RunnerEventType::ReceiveError { error }.into())
                    .map_err(|_| FatalError)?;
            }
        }

        Ok(())
    }

    async fn process_command(&mut self, command: RunnerCommand) -> Result<(), FatalError> {
        match command {
            RunnerCommand::QueryRoom {
                response_tx,
                room_id,
            } => {
                self.query_room(room_id, response_tx);
            }
            RunnerCommand::RequestToken {
                response_tx,
                room_id,
                client_parameters,
                room_parameters,
            } => {
                self.request_token(room_id, client_parameters, *room_parameters, response_tx)
                    .await;
            }
            RunnerCommand::ConnectSignaling {
                response_tx,
                token,
                url,
            } => {
                self.connect_signaling(url, token, response_tx).await?;
            }
            RunnerCommand::Send { message } => {
                self.send_websocket_message(message).await?;
            }
            RunnerCommand::Close => {
                self.disconnect().await?;
            }
            RunnerCommand::RoomServerAccess {
                response_tx,
                url,
                api_key,
            } => {
                self.disconnect().await?;
                let client = Client::new(url, api_key);
                self.client = client;
                let _ = response_tx.send(Ok(()));
            }
            RunnerCommand::SuspendReceive => self.receive_suspended = true,
            RunnerCommand::ResumeReceive => self.receive_suspended = false,
        }

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), FatalError> {
        if let Some(mut conn) = self.connection.take() {
            if let Err(e) = conn.close().await {
                self.event_tx
                    .send(
                        RunnerEventType::SendError {
                            error: e.to_string(),
                        }
                        .into(),
                    )
                    .map_err(|_| FatalError)?;
            }
            self.signaling_state_tx
                .send(SignalingState::Disconnect)
                .map_err(|_| FatalError)?;
            self.event_tx
                .send(RunnerEventType::Disconnected.into())
                .map_err(|_| FatalError)?;
        }
        Ok(())
    }

    fn query_room(
        &self,
        _room_id: RoomId,
        response_tx: tokio::sync::oneshot::Sender<RunnerResponse<bool>>,
    ) {
        let _ = response_tx.send(Ok(false));
    }

    async fn request_token(
        &self,
        room_id: RoomId,
        client_parameters: ClientParameters,
        room_parameters: Option<RoomParameters>,
        response_tx: tokio::sync::oneshot::Sender<RunnerResponse<RoomServerAccess>>,
    ) {
        let res = self
            .client
            .request_token(room_id, client_parameters, room_parameters)
            .await;
        match res {
            Ok(token) => {
                let _ = response_tx.send(Ok(token));
            }
            Err(e) => {
                let _ = response_tx.send(Err(anyhow::anyhow!(e)));
            }
        }
    }

    async fn connect_signaling(
        &mut self,
        url: Url,
        token: Token,
        response_tx: tokio::sync::oneshot::Sender<RunnerResponse<()>>,
    ) -> Result<(), FatalError> {
        self.disconnect().await?;

        let connection = match self.client.open_signaling_connection(url, token).await {
            Ok(con) => con,
            Err(e) => {
                let _ = response_tx.send(Err(e.into()));
                return Ok(());
            }
        };

        let _ = response_tx.send(Ok(()));
        self.signaling_state_tx
            .send(SignalingState::Connected)
            .map_err(|_| FatalError)?;
        self.event_tx
            .send(RunnerEventType::Connected.into())
            .map_err(|_| FatalError)?;

        self.connection.replace(connection);

        Ok(())
    }

    async fn send_websocket_message(&mut self, message: String) -> Result<(), FatalError> {
        let Some(conn) = self.connection.as_mut() else {
            log::debug!("Trying to send message while disconnected");
            self.event_tx
                .send(
                    RunnerEventType::SendError {
                        error: "Trying to send message while disconnected".to_string(),
                    }
                    .into(),
                )
                .map_err(|_| FatalError)?;
            return Ok(());
        };
        match conn.send_raw_message(&message).await {
            Ok(()) => {
                self.event_tx
                    .send(RunnerEventType::SendSuccess { message }.into())
                    .map_err(|_| FatalError)?;
            }
            Err(error) => {
                self.event_tx
                    .send(
                        RunnerEventType::SendError {
                            error: error.to_string(),
                        }
                        .into(),
                    )
                    .map_err(|_| FatalError)?;
            }
        }

        Ok(())
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{fmt::Debug, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use axum::extract::ws::Message;
use opentalk_orchestrator_shared::{
    Event, Metrics, RecorderEvent, RoomserverEvent, ServiceKind, TranscriptionEvent,
};
use tokio::time::Instant;
use url::Url;

use crate::storage::OrchestratorStorage;

const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Connection has been closed by the client")]
    ClosedByClient,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// The runner for a service instance
pub(crate) struct InstanceRunner {
    /// The websocket connection to the instance
    pub(crate) socket: axum::extract::ws::WebSocket,
    /// The HTTP address of the instance
    pub(crate) client_address: Url,
    /// The kind of service that the runner is managing
    pub(crate) kind: ServiceKind,
    /// The storage backend to use for state management
    pub(crate) storage: Arc<dyn OrchestratorStorage>,
}

impl InstanceRunner {
    pub(crate) fn new(
        socket: axum::extract::ws::WebSocket,
        client_address: Url,
        kind: ServiceKind,
        storage: Arc<dyn OrchestratorStorage>,
    ) -> Self {
        Self {
            socket,
            client_address,
            kind,
            storage,
        }
    }

    async fn set_service_metrics(&mut self, metrics: Metrics) -> Result<()> {
        self.storage
            .set_service_metrics(&self.client_address, self.kind, metrics)
            .await
            .context("Failed to set service metrics")
    }

    /// Run the event loop until the connection is closed by the service
    pub(crate) async fn run(mut self) -> Result<()> {
        let result = self.inner_run().await;

        if let Err(e) = self
            .storage
            .remove_instance(&self.client_address, self.kind)
            .await
        {
            tracing::error!(
                "Runner exited but failed to remove {} {} from storage: {e:?}",
                self.kind,
                self.client_address,
            )
        };

        if let Err(e) = result {
            match e {
                Error::ClosedByClient => return Ok(()),
                Error::Other(error) => return Err(error),
            }
        }

        Ok(())
    }

    async fn inner_run(&mut self) -> Result<(), Error> {
        let mut heartbeat =
            tokio::time::interval_at(Instant::now() + HEARTBEAT_TIMEOUT, HEARTBEAT_TIMEOUT);

        loop {
            tokio::select! {
                _ = heartbeat.tick() => {
                    tracing::error!("heartbeat timeout({HEARTBEAT_TIMEOUT:?}) triggered");
                    break;
                }
                msg = self.socket.recv() => {
                    heartbeat.reset();
                    self.handle_message(msg).await?}
            }
        }

        Ok(())
    }

    /// Handle an incoming websocket message
    async fn handle_message(
        &mut self,
        msg: Option<Result<Message, axum::Error>>,
    ) -> Result<(), Error> {
        let msg = msg
            .context("socket unexpectedly closed by client")?
            .context("error on websocket connection")?;

        let event: Event = match msg {
            Message::Close(close_frame) => {
                tracing::debug!(
                    "received close message {close_frame:?} for websocket connection ({})",
                    self.client_address
                );
                return Err(Error::ClosedByClient);
            }
            Message::Text(utf8_bytes) => {
                serde_json::from_str(utf8_bytes.as_str()).with_context(|| {
                    format!(
                        "failed to parse message for connection '{}':\n{utf8_bytes}",
                        self.client_address
                    )
                })?
            }
            Message::Binary(bytes) => serde_json::from_slice(bytes.iter().as_slice())
                .with_context(|| {
                    format!(
                        "failed parse to message for connection '{}'",
                        self.client_address
                    )
                })?,

            _ => {
                return Ok(());
            }
        };

        self.handle_service_event(event).await?;

        Ok(())
    }

    async fn handle_service_event(&mut self, event: Event) -> anyhow::Result<()> {
        match event {
            Event::Metrics(metrics) => self.set_service_metrics(metrics).await?,
            Event::Recorder(recorder_event) => self.handle_recorder_event(recorder_event).await?,
            Event::Roomserver(roomserver_event) => {
                self.handle_roomserver_event(roomserver_event).await?
            }
            Event::Transcription(transcription_event) => {
                self.handle_transcription_event(transcription_event).await?
            }
        }

        Ok(())
    }

    async fn handle_recorder_event(&mut self, recorder_event: RecorderEvent) -> anyhow::Result<()> {
        match recorder_event {
            RecorderEvent::RemoveRecording(resource) => {
                self.storage
                    .remove_service_resource(&self.client_address, resource.into())
                    .await?;
            }
        }

        Ok(())
    }

    async fn handle_roomserver_event(
        &mut self,
        roomserver_event: RoomserverEvent,
    ) -> anyhow::Result<()> {
        match roomserver_event {
            RoomserverEvent::RemoveRoom(room_id) => {
                self.storage
                    .remove_service_resource(&self.client_address, room_id.into())
                    .await?;
            }
        }

        Ok(())
    }

    async fn handle_transcription_event(
        &mut self,
        transcription_event: TranscriptionEvent,
    ) -> anyhow::Result<()> {
        match transcription_event {
            TranscriptionEvent::RemoveTranscription(resource) => {
                self.storage
                    .remove_service_resource(&self.client_address, resource.into())
                    .await?;
            }
        }

        Ok(())
    }
}

// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::fmt::Display;

use anyhow::Context as _;
use livekit::{RoomEvent, RoomOptions};
use opentalk_roomserver_client::api::event::Credentials;
use tokio::{
    runtime::Runtime,
    sync::{mpsc, watch},
};

use crate::app::signaling::livekit::handle::{LiveKitRunnerCommand, RunnerHandle, Status};

#[derive(Debug)]
enum State {
    Disconnected,
    Connected {
        room: livekit::Room,
        events: mpsc::UnboundedReceiver<RoomEvent>,
    },
}

impl State {
    async fn recv_livekit_event(&mut self) -> Option<RoomEvent> {
        if let State::Connected { events, .. } = self {
            events.recv().await
        } else {
            std::future::pending().await
        }
    }
}

#[derive(Debug)]
pub struct LiveKitRunner {
    egui_ctx: egui::Context,

    event_tx: mpsc::UnboundedSender<RoomEvent>,
    command_rx: mpsc::UnboundedReceiver<LiveKitRunnerCommand>,
    status_tx: watch::Sender<Status>,

    livekit: State,
}

impl LiveKitRunner {
    pub fn spawn(runtime: &Runtime, egui_ctx: egui::Context) -> RunnerHandle {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (status_tx, status_rx) = watch::channel(Status::Disconnected);
        let this = Self {
            event_tx,
            command_rx,
            status_tx,
            egui_ctx,

            livekit: State::Disconnected,
        };

        let join_handle = runtime.spawn(this.run());
        RunnerHandle::new(event_rx, command_tx, status_rx, join_handle)
    }

    async fn run(mut self) -> anyhow::Result<()> {
        loop {
            log::trace!("next livekit event");
            tokio::select! {
                msg = self.command_rx.recv() => {
                    if let Some(command) = msg {
                        self.handle_command(command).await?;
                        log::trace!("request repaint: handled livekit command");
                        self.egui_ctx.request_repaint();
                    } else {
                        log::trace!("livekit command channel dropped, exiting runner");
                        return Ok(());
                    }
                }

                msg = self.livekit.recv_livekit_event() => {
                    if let Some(event) = msg {
                        self.handle_event(event)?;
                        log::trace!("request repaint: handled livekit event");
                        self.egui_ctx.request_repaint();
                    } else {
                        log::trace!("livekit signaling connection dropped, exiting runner");
                        return Ok(());
                    }
                }
            }
        }
    }

    async fn handle_command(&mut self, command: LiveKitRunnerCommand) -> anyhow::Result<()> {
        match command {
            LiveKitRunnerCommand::Connect { credentials } => {
                self.handle_command_connect(credentials).await
            }
            LiveKitRunnerCommand::Disconnect => self.disconnect().await,
        }
    }

    async fn handle_command_connect(&mut self, credentials: Credentials) -> anyhow::Result<()> {
        let _ = warn_log_err(
            "failed to close LiveKit connection when using new token",
            self.disconnect().await,
        );

        self.connect(credentials.public_url, credentials.token)
            .await
    }

    fn handle_event(&mut self, event: RoomEvent) -> anyhow::Result<()> {
        if let RoomEvent::Disconnected { .. } = &event {
            self.livekit = State::Disconnected;
            self.status_tx
                .send(Status::Disconnected)
                .context("Failed to send disconnected status")?;
        }
        self.event_tx
            .send(event)
            .context("Failed to send LiveKit signaling event")?;
        Ok(())
    }

    async fn disconnect(&mut self) -> anyhow::Result<()> {
        if let State::Connected { room, .. } = &self.livekit {
            room.close()
                .await
                .context("Failed to close LiveKit room connection")?;
        }
        self.livekit = State::Disconnected;
        self.status_tx
            .send(Status::Disconnected)
            .context("Failed to send disconnected status")?;

        Ok(())
    }

    async fn connect(&mut self, url: String, token: String) -> anyhow::Result<()> {
        let (room, events) = livekit::Room::connect(&url, &token, RoomOptions::default())
            .await
            .context("Failed to connect to LiveKit room")?;
        self.status_tx
            .send(Status::Connected {
                room_name: room.name(),
            })
            .context("Failed to send connected status")?;
        self.livekit = State::Connected { room, events };

        Ok(())
    }
}

/// logs the error with a warning log and the given message. Handy for `map_err`.
fn warn_log_err<T, E: Display>(msg: &str, any_result: Result<T, E>) -> Result<T, E> {
    any_result.map_err(|e| {
        log::warn!("{msg}: {e}");
        e
    })
}

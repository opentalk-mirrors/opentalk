// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{pin::Pin, task::Context};

use anyhow::Context as _;
use livekit::RoomEvent;
use opentalk_roomserver_client::api::event::Credentials;
use tokio::{
    sync::{
        mpsc::{self, error::TryRecvError},
        watch,
    },
    task::JoinHandle,
};

#[derive(Debug)]
pub struct RunnerHandle {
    event_rx: mpsc::UnboundedReceiver<RoomEvent>,
    command_tx: mpsc::UnboundedSender<LiveKitRunnerCommand>,
    pub status_rx: watch::Receiver<Status>,

    join_handle: JoinHandle<anyhow::Result<()>>,
}

impl RunnerHandle {
    pub fn new(
        event_rx: mpsc::UnboundedReceiver<RoomEvent>,
        command_tx: mpsc::UnboundedSender<LiveKitRunnerCommand>,
        status_rx: watch::Receiver<Status>,
        join_handle: JoinHandle<anyhow::Result<()>>,
    ) -> Self {
        Self {
            event_rx,
            command_tx,
            status_rx,
            join_handle,
        }
    }

    pub fn connect(&self, token: Credentials) -> anyhow::Result<()> {
        self.command_tx
            .send(LiveKitRunnerCommand::Connect { credentials: token })?;

        Ok(())
    }

    pub fn disconnect(&self) -> anyhow::Result<()> {
        self.command_tx.send(LiveKitRunnerCommand::Disconnect)?;
        Ok(())
    }

    pub fn recv_event(&mut self) -> anyhow::Result<Option<RoomEvent>> {
        match self.event_rx.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(TryRecvError::Disconnected) => {
                // we poll the future every frame, no need for a waker.
                let waker = futures::task::noop_waker();
                let mut context = Context::from_waker(&waker);

                match Pin::new(&mut self.join_handle).poll(&mut context) {
                    std::task::Poll::Ready(Ok(Ok(()))) => {
                        anyhow::bail!("LiveKit Runner stopped unexpectedly")
                    }
                    std::task::Poll::Ready(Err(e)) => Err(e).context("LiveKit runner panicked"),
                    std::task::Poll::Ready(Ok(Err(e))) => Err(e).context("LiveKit runner errored"),
                    std::task::Poll::Pending => Ok(None),
                }
            }
            Err(TryRecvError::Empty) => Ok(None),
        }
    }
}

#[derive(Debug)]
pub enum Status {
    Disconnected,
    Connected { room_name: String },
}

impl Status {
    /// Returns `true` if the status is [`Disconnected`].
    ///
    /// [`Disconnected`]: Status::Disconnected
    #[must_use]
    pub fn is_disconnected(&self) -> bool {
        matches!(self, Self::Disconnected)
    }
}

#[derive(Debug)]
pub enum LiveKitRunnerCommand {
    Connect { credentials: Credentials },
    Disconnect,
}

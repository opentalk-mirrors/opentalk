// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use anyhow::{Context, Result};
use opentalk_roomserver_client::api::command::{BreakoutCommand, SignalingModuleCommand};
use opentalk_roomserver_types::{
    breakout::{
        BreakoutRoom,
        breakout_config::{BreakoutConfig as RoomserverBreakoutConfig, BreakoutRoomConfig},
        breakout_id::BreakoutId,
    },
    room_kind::RoomKind,
};
use opentalk_types::common::time::Timestamp;

use crate::Room;

#[derive(Debug, Clone)]
pub struct BreakoutState {
    pub current_room: RoomKind,
    pub config: Option<BreakoutConfig>,
}

impl Default for BreakoutState {
    fn default() -> Self {
        Self {
            current_room: RoomKind::Main,
            config: None,
        }
    }
}

impl BreakoutState {
    pub(crate) fn set_config(&mut self, config: BreakoutConfig) {
        self.config = Some(config);
    }
}

#[derive(Debug, Clone)]
pub struct BreakoutConfig {
    pub rooms: Vec<BreakoutRoom>,
    pub expires_at: Option<Timestamp>,
    pub assignment: Option<BreakoutId>,
}

pub struct BreakoutApi<'a> {
    pub(super) room: &'a mut Room,
}

impl BreakoutApi<'_> {
    /// Start a new breakout session with the given rooms and duration
    pub async fn start_breakout(
        &mut self,
        rooms: Vec<BreakoutRoomConfig>,
        duration: Option<Duration>,
    ) -> Result<()> {
        self.room
            .signaling
            .send(
                &SignalingModuleCommand::Breakout(BreakoutCommand::Start(
                    RoomserverBreakoutConfig { rooms, duration },
                ))
                .into(),
            )
            .await
            .context("Failed to send breakout start command")
    }

    /// Stop the current breakout session
    pub async fn stop_breakout(&mut self, delay: Option<Duration>) -> Result<()> {
        self.room
            .signaling
            .send(&SignalingModuleCommand::Breakout(BreakoutCommand::Stop { delay }).into())
            .await
            .context("Failed to send breakout stop command")
    }

    /// Switch to the main room or a different breakout room
    pub async fn switch_room(&mut self, new_room: RoomKind) -> Result<()> {
        self.room
            .signaling
            .send(&SignalingModuleCommand::Breakout(BreakoutCommand::SwitchRoom(new_room)).into())
            .await
            .context("Failed to send breakout switch command")
    }

    /// Get the current room of the client
    pub fn current_room(&self) -> RoomKind {
        self.room.breakout_state.current_room
    }

    /// Get the current breakout configuration
    pub fn breakout_config(&self) -> Option<BreakoutConfig> {
        self.room.breakout_state.config.clone()
    }
}

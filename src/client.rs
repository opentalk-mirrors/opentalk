use std::collections::HashMap;

use anyhow::{Context, Result};
use types_common::users::DisplayName;

use crate::config::ClientConfig;
use crate::http::HttpClient;
use crate::signaling::Signaling;
use crate::{ControllerConfig, Room, WaitingRoomState};

pub struct OpenTalkClient {
    controller_config: ControllerConfig,
    http: HttpClient,
}

impl OpenTalkClient {
    pub async fn create(config: ClientConfig) -> Result<Self> {
        let http = HttpClient::create(config.auth).await?;

        Ok(Self {
            controller_config: config.controller,
            http,
        })
    }

    pub async fn connect(
        &self,
        display_name: DisplayName,
        room_id: &str,
        use_livekit: bool,
    ) -> Result<JoinResult> {
        let signaling = Signaling::connect(
            &self.http,
            &self.controller_config,
            display_name,
            room_id,
            None,
        )
        .await.context("Failed to connect to signaling")?;

        let mut room = Room {
            use_livekit,
            waiting_room_state: WaitingRoomState::None,
            signaling,
            livekit_room: None,
            livekit_events: None,
            participants: HashMap::new(),
        };

        room.join().await?;

        if room.is_in_waiting_room() {
            return Ok(JoinResult::InWaitingRoom(room));
        } else {
            return Ok(JoinResult::InRoom(room));
        }
    }
}

pub enum JoinResult {
    InRoom(Room),
    InWaitingRoom(Room),
}

impl From<JoinResult> for Room {
    fn from(join_result: JoinResult) -> Self {
        match join_result {
            JoinResult::InRoom(room) => room,
            JoinResult::InWaitingRoom(room) => room,
        }
    }
}

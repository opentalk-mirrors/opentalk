use std::collections::HashMap;

use anyhow::Result;
use types_common::users::DisplayName;

use crate::http::HttpClient;
use crate::settings::Settings;
use crate::signaling::Signaling;
use crate::{Room, WaitingRoomState};

pub struct OpenTalkClient {
    settings: Settings,
    http: HttpClient,
}

impl OpenTalkClient {
    pub async fn create(settings: Settings) -> Result<Self> {
        let http = HttpClient::discover(&settings.auth).await?;

        Ok(Self { settings, http })
    }

    pub async fn connect(
        &self,
        display_name: DisplayName,
        room_id: &str,
        use_livekit: bool,
    ) -> Result<JoinResult> {
        let signaling = Signaling::connect(
            &self.http,
            &self.settings.controller,
            display_name,
            room_id,
            None,
        )
        .await?;

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

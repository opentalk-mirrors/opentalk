use anyhow::{Context, Result};
use types::common::{
    auth::TicketToken,
    call_in::{CallInId, CallInPassword},
    rooms::{BreakoutRoomId, RoomId, RoomPassword},
    users::DisplayName,
};

use crate::{
    ControllerConfig, config::ClientConfig, http::HttpClient, room::Room, signaling::Signaling,
};

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

    pub async fn connect_authorized(
        &self,
        display_name: DisplayName,
        room_id: RoomId,
        breakout_id: Option<BreakoutRoomId>,
        use_livekit: bool,
    ) -> Result<JoinResult> {
        let ticket = self
            .http
            .start_authorized(&self.controller_config, room_id, breakout_id)
            .await
            .context("Failed to send start HTTP request")?;

        self.connect_common(ticket, display_name, use_livekit).await
    }

    pub async fn connect_invited(
        &self,
        room_id: RoomId,
        breakout_id: Option<BreakoutRoomId>,
        invite_code: String,
        password: Option<RoomPassword>,
        display_name: DisplayName,
        use_livekit: bool,
    ) -> Result<JoinResult> {
        let ticket = self
            .http
            .start_invited(
                &self.controller_config,
                room_id,
                breakout_id,
                invite_code,
                password,
            )
            .await
            .context("Failed to send start-call-in HTTP request")?;

        self.connect_common(ticket, display_name, use_livekit).await
    }

    pub async fn connect_call_in(
        &self,
        id: CallInId,
        pin: CallInPassword,
        display_name: DisplayName,
        use_livekit: bool,
    ) -> Result<JoinResult> {
        let ticket = self
            .http
            .start_call_in(&self.controller_config, id, pin)
            .await
            .context("Failed to send start-call-in HTTP request")?;

        self.connect_common(ticket, display_name, use_livekit).await
    }

    async fn connect_common(
        &self,
        ticket: TicketToken,
        display_name: DisplayName,
        use_livekit: bool,
    ) -> Result<JoinResult> {
        let signaling = Signaling::connect(ticket, &self.controller_config, display_name)
            .await
            .context("Failed to connect to signaling")?;

        let mut room = Room::new(use_livekit, signaling);

        room.join().await?;

        if room.is_in_waiting_room() {
            Ok(JoinResult::InWaitingRoom(room))
        } else {
            Ok(JoinResult::InRoom(room))
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

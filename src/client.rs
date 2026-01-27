// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result};
use opentalk_roomserver_client::api::signaling::SignalingConnection;
use opentalk_roomserver_types::breakout::breakout_id::BreakoutId;
use types::{
    api::v1::rooms::{RoomResource, by_room_id::RoomserverStartResponseBody},
    common::{
        call_in::{CallInId, CallInPassword},
        rooms::{RoomId, RoomPassword},
        users::DisplayName,
    },
};

use crate::{
    config::{BaseUrl, ClientConfig},
    http::{CreateRoomError, GetRoomError, HttpClient},
    room::Room,
};

pub struct OpenTalkClient {
    controller_url: BaseUrl,
    pub(crate) http: HttpClient,
}

impl OpenTalkClient {
    /// Creates a new [`OpenTalkClient`]
    pub async fn create(config: ClientConfig) -> Result<Self> {
        let http = HttpClient::create(config.auth).await?;

        Ok(Self {
            controller_url: config.controller,
            http,
        })
    }

    /// Connect as authenticated opentalk user
    pub async fn connect_as_user(
        &self,
        display_name: Option<DisplayName>,
        room_id: RoomId,
        #[cfg(feature = "livekit")] use_livekit: bool,
    ) -> Result<JoinResult> {
        let response = self
            .http
            .start_as_user(&self.controller_url, room_id, display_name)
            .await
            .context("Failed to send start HTTP request")?;

        self.connect_common(
            response,
            #[cfg(feature = "livekit")]
            use_livekit,
        )
        .await
    }

    /// Connect using an invite code, does not require any authentication
    pub async fn connect_invited(
        &self,
        room_id: RoomId,
        invite_code: String,
        password: Option<RoomPassword>,
        display_name: DisplayName,
        #[cfg(feature = "livekit")] use_livekit: bool,
    ) -> Result<JoinResult> {
        let response = self
            .http
            .start_invited(
                &self.controller_url,
                room_id,
                invite_code,
                password,
                display_name,
            )
            .await
            .context("Failed to send start-call-in HTTP request")?;

        self.connect_common(
            response,
            #[cfg(feature = "livekit")]
            use_livekit,
        )
        .await
    }

    /// Connect as call-in service. Requires service authentication
    pub async fn connect_call_in(
        &self,
        id: CallInId,
        pin: CallInPassword,
        display_name: DisplayName,
        #[cfg(feature = "livekit")] use_livekit: bool,
    ) -> Result<JoinResult> {
        let response = self
            .http
            .start_call_in(&self.controller_url, id, pin, display_name)
            .await
            .context("Failed to send start-call-in HTTP request")?;

        self.connect_common(
            response,
            #[cfg(feature = "livekit")]
            use_livekit,
        )
        .await
    }

    /// Connect as authenticated opentalk user
    pub async fn connect_recorder(
        &self,
        room_id: RoomId,
        breakout_id: Option<BreakoutId>,
        #[cfg(feature = "livekit")] use_livekit: bool,
    ) -> Result<JoinResult> {
        let response = self
            .http
            .start_as_recorder(&self.controller_url, room_id, breakout_id)
            .await
            .context("Failed to send start HTTP request")?;

        self.connect_common(
            response,
            #[cfg(feature = "livekit")]
            use_livekit,
        )
        .await
    }

    async fn connect_common(
        &self,
        response: RoomserverStartResponseBody,
        #[cfg(feature = "livekit")] use_livekit: bool,
    ) -> Result<JoinResult> {
        let connection = SignalingConnection::connect(
            response
                .roomserver_address
                .parse()
                .context("Failed to parse roomserver address to URL")?,
            response.token,
        )
        .await
        .context("Failed to connect to roomserver signaling")?;

        let mut room = Room::new(
            connection,
            #[cfg(feature = "livekit")]
            use_livekit,
        );

        room.join().await?;

        if room.is_in_waiting_room() {
            Ok(JoinResult::InWaitingRoom(room))
        } else {
            Ok(JoinResult::InRoom(room))
        }
    }

    pub async fn get_rooms(&self, max: usize) -> Result<Vec<RoomResource>, GetRoomError> {
        self.http.get_rooms(&self.controller_url, max).await
    }

    pub async fn create_room(
        &self,
        password: Option<RoomPassword>,
        enable_sip: bool,
        waiting_room: bool,
        e2e_encryption: bool,
    ) -> Result<RoomResource, CreateRoomError> {
        self.http
            .create_room(
                &self.controller_url,
                password,
                enable_sip,
                waiting_room,
                e2e_encryption,
            )
            .await
    }
}

pub enum JoinResult {
    InRoom(Room),
    InWaitingRoom(Room),
}

impl From<JoinResult> for Room {
    fn from(join_result: JoinResult) -> Self {
        match join_result {
            JoinResult::InRoom(room) | JoinResult::InWaitingRoom(room) => room,
        }
    }
}

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
        rooms::{GuestAccess, RoomId, RoomPassword},
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
    ) -> Result<JoinResult> {
        let response = self
            .http
            .start_as_user(&self.controller_url, room_id, display_name)
            .await
            .context("Failed to send start HTTP request")?;

        self.connect_common(response).await
    }

    /// Connect using an invite code, does not require any authentication
    pub async fn connect_invited(
        &self,
        room_id: RoomId,
        invite_code: String,
        password: Option<RoomPassword>,
        display_name: DisplayName,
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

        self.connect_common(response).await
    }

    /// Connect as call-in service. Requires ApiKey authentication
    ///
    /// Flow for call-in is separated into two steps since services will need to verify the input
    /// using the start endpoint, without immediately joining the conference.
    pub async fn start_call_in(
        &self,
        id: CallInId,
        pin: CallInPassword,
        display_name: DisplayName,
    ) -> Result<CallInConnectToken> {
        let start_response = self
            .http
            .start_call_in(&self.controller_url, id, pin, display_name)
            .await
            .context("Failed to send start-call-in HTTP request")?;

        Ok(CallInConnectToken {
            response: start_response,
        })
    }

    /// Connect as call-in service. Requires a [`CallInConnectToken`] from [`Self::start_call_in`]
    pub async fn connect_call_in(
        &self,
        CallInConnectToken { response }: CallInConnectToken,
    ) -> Result<JoinResult> {
        self.connect_common(response).await
    }

    /// Connect as recording service
    pub async fn connect_recorder(
        &self,
        room_id: RoomId,
        breakout_id: Option<BreakoutId>,
    ) -> Result<JoinResult> {
        let response = self
            .http
            .start_as_recorder(&self.controller_url, room_id, breakout_id)
            .await
            .context("Failed to send start HTTP request")?;

        self.connect_common(response).await
    }

    /// Connect as transcription service
    pub async fn connect_transcription(
        &self,
        room_id: RoomId,
        breakout_id: Option<BreakoutId>,
    ) -> Result<JoinResult> {
        let response = self
            .http
            .start_as_transcription(&self.controller_url, room_id, breakout_id)
            .await
            .context("Failed to send start HTTP request")?;

        self.connect_common(response).await
    }

    async fn connect_common(&self, response: RoomserverStartResponseBody) -> Result<JoinResult> {
        let connection = SignalingConnection::connect(
            response
                .roomserver_address
                .parse()
                .context("Failed to parse roomserver address to URL")?,
            response.token,
        )
        .await
        .context("Failed to connect to roomserver signaling")?;

        let mut room = Room::new(connection);

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
        guest_access: GuestAccess,
        e2e_encryption: bool,
    ) -> Result<RoomResource, CreateRoomError> {
        self.http
            .create_room(
                &self.controller_url,
                password,
                enable_sip,
                waiting_room,
                guest_access,
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

/// Intermediate type used by [`OpenTalkClient::start_call_in`] & [`OpenTalkClient::connect_call_in`]
#[derive(Debug)]
pub struct CallInConnectToken {
    response: RoomserverStartResponseBody,
}

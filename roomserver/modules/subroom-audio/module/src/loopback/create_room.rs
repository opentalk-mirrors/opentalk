// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use livekit_api::services::room::{CreateRoomOptions, RoomClient};
use opentalk_roomserver_types_subroom_audio::event::SubroomAudioError;

use crate::loopback::SubroomAudioLoopback;

pub async fn create_room(
    livekit_client: Arc<RoomClient>,
    livekit_room_id: String,
) -> Result<SubroomAudioLoopback, SubroomAudioError> {
    let room = livekit_client
        .create_room(&livekit_room_id, CreateRoomOptions::default())
        .await
        .map_err(|err| {
            tracing::error!("failed to create room: {}", err);
            SubroomAudioError::LivekitUnavailable
        })?;
    tracing::debug!("LiveKit audio subroom created: {}", room.name);
    Ok(SubroomAudioLoopback::RoomCreated)
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use livekit_api::services::room::RoomClient;
use opentalk_roomserver_types_subroom_audio::event::SubroomAudioError;

use crate::loopback::SubroomAudioLoopback;

pub async fn destroy_room(
    livekit_client: Arc<RoomClient>,
    livekit_room_id: String,
) -> Result<SubroomAudioLoopback, SubroomAudioError> {
    livekit_client
        .delete_room(&livekit_room_id)
        .await
        .map_err(|err| {
            tracing::debug!(
                "failed to remove livekit whisper room for id {}: {}",
                livekit_room_id,
                err
            );
            SubroomAudioError::LivekitUnavailable
        })?;

    tracing::debug!("LiveKit audio subroom destroyed: {}", livekit_room_id);
    Ok(SubroomAudioLoopback::RoomDestroyed)
}

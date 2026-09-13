// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use livekit_api::services::room::{CreateRoomOptions, RoomClient};
use opentalk_roomserver_types_livekit::LiveKitError;

use crate::loopback::LiveKitLoopback;

pub async fn create_room(
    livekit_client: Arc<RoomClient>,
    subroom_id: String,
) -> Result<LiveKitLoopback, LiveKitError> {
    let room = livekit_client
        .create_room(&subroom_id, CreateRoomOptions::default())
        .await
        .map_err(|err| {
            tracing::error!(error = ?err, "failed to create room");
            LiveKitError::LivekitUnavailable
        })?;
    tracing::debug!("LiveKit room created: {}", room.name);
    Ok(LiveKitLoopback::RoomCreated)
}

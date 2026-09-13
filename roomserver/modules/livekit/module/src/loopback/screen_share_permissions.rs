// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use livekit_api::services::room::RoomClient;
use livekit_protocol::TrackSource;
use opentalk_roomserver_types::connection_id::ConnectionId;
use opentalk_roomserver_types_livekit::LiveKitError;
use opentalk_types_signaling::ParticipantId;

use super::{LiveKitLoopback, update_participants_permission};
use crate::build_livekit_participant_id;

pub async fn set_screenshare_permissions(
    livekit_client: Arc<RoomClient>,
    room: String,
    sender: ParticipantId,
    participants: BTreeMap<ParticipantId, BTreeSet<ConnectionId>>,
    grant: bool,
) -> Result<LiveKitLoopback, LiveKitError> {
    let affected_participants =
        affected_participants(&livekit_client, &room, &participants).await?;

    update_participants_permission(
        &livekit_client,
        affected_participants,
        &[
            TrackSource::ScreenShare as i32,
            TrackSource::ScreenShareAudio as i32,
        ],
        grant,
        &room,
    )
    .await;

    Ok(LiveKitLoopback::ScreenSharePermissionsUpdated {
        sender,
        grant,
        participants: participants.keys().copied().collect(),
    })
}

async fn affected_participants(
    livekit_client: &Arc<RoomClient>,
    room: &str,
    participants: &BTreeMap<ParticipantId, BTreeSet<ConnectionId>>,
) -> Result<Vec<livekit_protocol::ParticipantInfo>, LiveKitError> {
    let mut all_participants = livekit_client
        .list_participants(room)
        .await
        .map_err(|err| {
            tracing::error!("failed to query participants, {err}");
            LiveKitError::LivekitUnavailable
        })?;
    let livekit_participant_ids = participants
        .iter()
        .flat_map(|(p_id, c_ids)| {
            c_ids
                .iter()
                .map(|c_id| build_livekit_participant_id(*p_id, *c_id))
        })
        .collect::<BTreeSet<String>>();

    all_participants.retain(|p| livekit_participant_ids.contains(&p.identity));
    Ok(all_participants)
}

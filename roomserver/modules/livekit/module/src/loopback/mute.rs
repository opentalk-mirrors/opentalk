// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use futures::{StreamExt as _, stream};
use livekit_api::services::room::RoomClient;
use livekit_protocol::TrackSource;
use opentalk_roomserver_types_livekit::ParticipantsMuted;
use opentalk_types_signaling::ParticipantId;
use tracing::{Instrument as _, debug_span};

use crate::{LiveKitConnection, PARALLEL_UPDATES};

pub async fn mute_participants(
    livekit_client: Arc<RoomClient>,
    sender: Option<ParticipantId>,
    participant_connections: Vec<LiveKitConnection>,
) -> ParticipantsMuted {
    let muted_participants = stream::iter(participant_connections).map(
        |LiveKitConnection { participant_id, livekit_participant_id, livekit_room: room}| {
            let client = Arc::clone(&livekit_client);
            async move {
                let mute_span = debug_span!("mute", livekit_participant_id);
                let result = mute(&client, &room, &livekit_participant_id)
                    .instrument(mute_span.clone())
                    .await;
                match result {
                    Ok(true) => Some(participant_id),
                    Ok(false) => None,
                    Err(e) => {
                        // The participant might not have a microphone or already left the meeting.
                        tracing::debug!(parent: &mute_span, "failed to mute participant connection '{participant_id}': {e}");
                        None
                    }
                }
            }
        },
    )
    .buffer_unordered(PARALLEL_UPDATES)
    .filter_map(|participant_id| async move { participant_id })
    .collect()
    .await;

    ParticipantsMuted {
        sender,
        participants: muted_participants,
    }
}

async fn mute(
    livekit_client: &Arc<RoomClient>,
    room: &str,
    livekit_participant_id: &str,
) -> anyhow::Result<bool> {
    let livekit_participant = livekit_client
        .get_participant(room, livekit_participant_id)
        .await?;

    let mut muted_any = false;
    for track in livekit_participant.tracks {
        if track.source != TrackSource::Microphone as i32 {
            // Don't mute non-microphone tracks
            tracing::trace!("Skipped muting track, not a microphone.");
            continue;
        }

        if track.muted {
            tracing::trace!("Skipped muting track, already muted.");
            continue;
        }

        livekit_client
            .mute_published_track(room, livekit_participant_id, &track.sid, true)
            .await?;
        tracing::debug!("Muted participant connection");
        muted_any = true;
    }

    Ok(muted_any)
}

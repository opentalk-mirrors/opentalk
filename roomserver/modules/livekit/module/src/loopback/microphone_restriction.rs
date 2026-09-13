// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use livekit_api::services::room::RoomClient;
use livekit_protocol::{ParticipantInfo, TrackSource};
use opentalk_roomserver_types::connection_id::ConnectionId;
use opentalk_roomserver_types_livekit::{
    MicrophoneRestrictionError, MicrophoneRestrictionErrorKind, MicrophoneRestrictionState,
};
use opentalk_types_signaling::ParticipantId;

use super::update_participants_permission;
use crate::build_livekit_participant_id;

#[tracing::instrument(skip(livekit_client), level = "debug")]
pub async fn update_restricted_microphones(
    livekit_client: Arc<RoomClient>,
    room: String,
    sender: ParticipantId,
    state: MicrophoneRestrictionState,
    participant_connections: BTreeMap<ParticipantId, BTreeSet<ConnectionId>>,
) -> Result<MicrophoneRestrictionState, MicrophoneRestrictionError> {
    let participants =
        affected_participants(&livekit_client, &room, &state, participant_connections)
            .await
            .map_err(|error| MicrophoneRestrictionError { sender, error })?;

    tracing::debug!("update microphone restrictions");
    update_participants_permission(
        &livekit_client,
        participants.restricted,
        &[TrackSource::Microphone as i32],
        false,
        &room,
    )
    .await;

    update_participants_permission(
        &livekit_client,
        participants.allowed,
        &[TrackSource::Microphone as i32],
        true,
        &room,
    )
    .await;

    Ok(state)
}

pub struct AffectedParticipants {
    allowed: Vec<ParticipantInfo>,
    restricted: Vec<ParticipantInfo>,
}

async fn affected_participants(
    livekit_client: &Arc<RoomClient>,
    room: &str,
    state: &MicrophoneRestrictionState,
    participant_connections: BTreeMap<ParticipantId, BTreeSet<ConnectionId>>,
) -> Result<AffectedParticipants, MicrophoneRestrictionErrorKind> {
    // get all participants connected to livekit
    let mut participants = livekit_client
        .list_participants(room)
        .await
        .map_err(|err| {
            tracing::error!("Failed to query participants, {err}");
            MicrophoneRestrictionErrorKind::LivekitUnavailable
        })?;
    tracing::trace!("Participants in livekit room: {:?}", participants);

    // filter unrestricted participants since they must not be affected by the restrictions.
    if let MicrophoneRestrictionState::Enabled {
        unrestricted_participants,
    } = state
    {
        let mut allowed_ids = BTreeSet::new();
        for unrestricted_participant in unrestricted_participants {
            if let Some(connections) = participant_connections.get(unrestricted_participant) {
                for connection_id in connections {
                    allowed_ids.insert(build_livekit_participant_id(
                        *unrestricted_participant,
                        *connection_id,
                    ));
                }
            }
        }
        let allowed = participants
            .extract_if(.., |part| allowed_ids.contains(&part.identity))
            .collect();
        Ok(AffectedParticipants {
            allowed,
            restricted: participants,
        })
    } else {
        Ok(AffectedParticipants {
            allowed: participants,
            restricted: Vec::new(),
        })
    }
}

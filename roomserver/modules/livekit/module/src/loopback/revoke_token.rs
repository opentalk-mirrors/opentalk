// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::BTreeSet, sync::Arc};

use livekit_api::services::{ServiceError, TwirpError, TwirpErrorCode, room::RoomClient};
use opentalk_roomserver_types::connection_id::ConnectionId;
use opentalk_roomserver_types_livekit::LiveKitError;
use opentalk_types_signaling::ParticipantId;

use crate::loopback::LiveKitLoopback;

pub async fn revoke_token(
    livekit_client: Arc<RoomClient>,
    participant_id: ParticipantId,
    connection_id: ConnectionId,
    room: String,
    token_identities: BTreeSet<String>,
) -> Result<LiveKitLoopback, LiveKitError> {
    let mut revoked_identities = BTreeSet::new();
    for identity in token_identities {
        match livekit_client.remove_participant(&room, &identity).await {
            Ok(_) => {
                revoked_identities.insert(identity);
            }
            // This can happen when the user does not disconnect properly, e.g. when closing the
            // browser tab and LiveKit already removed the participant.
            Err(ServiceError::Twirp(TwirpError::Twirp(twirp_err)))
                if twirp_err.code == TwirpErrorCode::NOT_FOUND =>
            {
                tracing::debug!("Participant already removed from LiveKit");
                revoked_identities.insert(identity);
            }
            Err(err) => {
                tracing::error!("Failed to revoke token: {err}");
            }
        }
    }
    Ok(LiveKitLoopback::NoteRevokedTokens {
        participant_id,
        connection_id,
        token_identities: revoked_identities,
    })
}

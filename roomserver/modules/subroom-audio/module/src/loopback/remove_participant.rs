// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use livekit_api::services::room::RoomClient;
use opentalk_roomserver_types_subroom_audio::event::SubroomAudioError;

use crate::loopback::SubroomAudioLoopback;

pub async fn remove_participant(
    livekit_client: Arc<RoomClient>,
    whisper_id: String,
    participant_id: String,
) -> Result<SubroomAudioLoopback, SubroomAudioError> {
    // This errors with a `not_found` when the participant already left or never joined. The
    // frontend client might leave the livekit room with the participant before we attempt to
    // remove them. We can't match the livekit error because the inner error types are not
    // exposed. Simply ignore potential errors for now.
    let result = livekit_client
        .remove_participant(&whisper_id, &participant_id)
        .await;

    if result.is_err() {
        tracing::debug!(
            "Failed to remove participant {participant_id} from livekit room {whisper_id}: {:?}",
            result.err()
        );
    }

    Ok(SubroomAudioLoopback::ParticipantRemoved)
}

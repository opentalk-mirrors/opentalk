// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_signaling::signaling_module::InternalCommand;
use opentalk_types_signaling::ParticipantId;

/// Update the shared folder access of a participant based on their current role
pub struct UpdateSharedFolder {
    /// The affected participant
    pub participant_id: ParticipantId,
}

impl InternalCommand for UpdateSharedFolder {}

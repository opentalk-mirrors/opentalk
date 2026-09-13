// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use opentalk_types_signaling::ParticipantId;

use crate::event::LiveKitEvent;

/// The livekit command variants
#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum LiveKitCommand {
    /// Indicates that a new Access Token is requested
    CreateNewAccessToken,

    /// Allows the specified participants to share their screens
    GrantScreenSharePermission {
        /// The participants that get granted screen sharing permissions
        participants: BTreeSet<ParticipantId>,
    },

    /// Revokes the permission to share their screen
    RevokeScreenSharePermission {
        /// The participants
        participants: BTreeSet<ParticipantId>,
    },

    /// Request a new livekit access token that cannot publish and is hidden to other participants
    RequestPopoutStreamAccessToken,
}

impl CreateReplica<LiveKitEvent> for LiveKitCommand {
    fn replicate(&self) -> Option<LiveKitEvent> {
        None
    }
}

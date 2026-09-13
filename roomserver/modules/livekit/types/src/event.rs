// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_types_signaling::ParticipantId;

use crate::{Credentials, error::LiveKitError};

/// The events emitted for livekit
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum LiveKitEvent {
    /// The credentials for a client to use livekit
    Credentials(Credentials),

    /// A livekit access token that cannot publish and is hidden to other participants
    PopoutStreamAccessToken {
        /// The token
        token: String,
    },

    /// LiveKit permissions have been updated.
    ///
    /// This event is the response to [`LiveKitCommand::RevokeScreenSharePermission`]
    /// and [`LiveKitCommand::GrantScreenSharePermission`] and only received by the
    /// moderator who issued the command. The participant who was the target of the
    /// command will be notified by the LiveKit server.
    ///
    /// [`LiveKitCommand::RevokeScreenSharePermission`]: crate::command::LiveKitCommand::RevokeScreenSharePermission
    /// [`LiveKitCommand::GrantScreenSharePermission`]: crate::command::LiveKitCommand::GrantScreenSharePermission
    ScreenSharePermissionsUpdated {
        /// `true` if screen share permissions where granted, `false` otherwise.
        grant: bool,
        /// The participants who received a screen share permission change.
        participants: BTreeSet<ParticipantId>,
    },

    /// The last message couldn't be processed since an unexpected error occurred.
    Error(LiveKitError),
}

impl From<LiveKitError> for LiveKitEvent {
    fn from(error: LiveKitError) -> Self {
        Self::Error(error)
    }
}

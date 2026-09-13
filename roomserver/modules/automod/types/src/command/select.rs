// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// Moderator command, select the speaker
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "how")]
pub enum Select {
    /// Revoke speaker status if exists, do no select a new one
    None,

    /// Select a random speaker
    Random,

    /// Advance the moderation depending on the selection strategy.
    /// Can just unset the current speaker if selection strategy is nomination
    Next,

    /// Select a specific participant
    Specific {
        /// The participant to be selected
        participant: ParticipantId,

        /// If true the selected participant will not be removed from either the allow- or playlist
        keep_in_remaining: bool,
    },
}

// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::{time::Timestamp, users::DisplayName};
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// Represents a banned participant
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BannedParticipant {
    /// The display name of the banned participant
    ///
    /// Needs to be saved because a banned participant might have never joined the conference, we
    /// can't ensure that the participant id can be resolved by the frontend.
    pub display_name: DisplayName,

    /// The avatar url of the banned participant
    pub avatar_url: String,

    /// The moderator that banned the participant
    pub banned_by: ParticipantId,

    /// The time that the participant got banned
    pub banned_at: Timestamp,
}

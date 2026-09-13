// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Frontend data for `livekit` namespace

use std::collections::BTreeSet;

use opentalk_types_signaling::ParticipantId;

/// Moderation module state that is visible only to moderators
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MicrophoneRestrictionState {
    /// The force mute state is disabled, participants are allowed to unmute
    #[default]
    Disabled,
    /// The force mute state is enabled, only the participants part of `unrestricted_participants`
    /// are allowed to unmute
    Enabled {
        /// The list of participants that are still allowed to unmute
        unrestricted_participants: BTreeSet<ParticipantId>,
    },
}

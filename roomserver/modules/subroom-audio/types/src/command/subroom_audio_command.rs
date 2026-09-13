// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use super::participant_targets::ParticipantTargets;
use crate::{event::SubroomAudioEvent, whisper_id::WhisperId};

/// Commands for the subroom audio whisper functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum SubroomAudioCommand {
    /// Create a new whisper group
    CreateWhisperGroup {
        /// A list of invited participants
        participant_ids: BTreeSet<ParticipantId>,
    },

    /// Invite participants to join an existing whisper group
    InviteToWhisperGroup(ParticipantTargets),

    /// Kick whisper participants from the group
    KickWhisperParticipants(ParticipantTargets),

    /// Accept a whisper invite
    AcceptWhisperInvite {
        /// The targeted whisper group
        whisper_id: WhisperId,
    },

    /// Decline a whisper invite
    DeclineWhisperInvite {
        /// The targeted whisper group
        whisper_id: WhisperId,
    },

    /// Leave a whisper group
    LeaveWhisperGroup {
        /// The targeted whisper group
        whisper_id: WhisperId,
    },
}

impl CreateReplica<SubroomAudioEvent> for SubroomAudioCommand {
    fn replicate(&self) -> Option<SubroomAudioEvent> {
        match self {
            SubroomAudioCommand::InviteToWhisperGroup(targets) => {
                Some(SubroomAudioEvent::ParticipantsInvited {
                    whisper_id: targets.whisper_id,
                    participant_ids: targets.participant_ids.iter().copied().collect(),
                })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use opentalk_types_signaling::ParticipantId;
    use serde_json::json;

    use super::SubroomAudioCommand;
    use crate::{command::ParticipantTargets, whisper_id::WhisperId};

    #[test]
    fn create_whisper_group() {
        let json = json!({
            "action": "create_whisper_group",
            "participant_ids": [
                "00000000-0000-0000-0000-000000000000",
                "00000000-0000-0000-0000-000000000001",
                "00000000-0000-0000-0000-000000000002"
            ]

        });

        match serde_json::from_value(json).unwrap() {
            SubroomAudioCommand::CreateWhisperGroup {
                participant_ids: participants,
            } => {
                assert_eq!(
                    participants,
                    BTreeSet::from([
                        ParticipantId::from_u128(0),
                        ParticipantId::from_u128(1),
                        ParticipantId::from_u128(2)
                    ])
                );
            }
            unexpected => panic!("Expected create_whisper_group message, got: {unexpected:?}"),
        }
    }

    #[test]
    fn invite_to_whisper_group() {
        let json = json!({
            "action": "invite_to_whisper_group",
            "whisper_id": "00000000-0000-0000-0000-000000000000",
            "participant_ids": [
                "00000000-0000-0000-0000-000000000000",
                "00000000-0000-0000-0000-000000000001",
                "00000000-0000-0000-0000-000000000002"
            ]

        });

        match serde_json::from_value(json).unwrap() {
            SubroomAudioCommand::InviteToWhisperGroup(ParticipantTargets {
                whisper_id,
                participant_ids: participants,
            }) => {
                assert_eq!(whisper_id, WhisperId::nil());
                assert_eq!(
                    participants,
                    BTreeSet::from([
                        ParticipantId::from_u128(0),
                        ParticipantId::from_u128(1),
                        ParticipantId::from_u128(2)
                    ])
                );
            }
            unexpected => panic!("Expected invite_to_whisper_group message, got: {unexpected:?}"),
        }
    }

    #[test]
    fn kick_whisper_participants() {
        let json = json!({
            "action": "kick_whisper_participants",
            "whisper_id": "00000000-0000-0000-0000-000000000000",
            "participant_ids": [
                "00000000-0000-0000-0000-000000000000",
                "00000000-0000-0000-0000-000000000001",
                "00000000-0000-0000-0000-000000000002"
            ]

        });

        match serde_json::from_value(json).unwrap() {
            SubroomAudioCommand::KickWhisperParticipants(ParticipantTargets {
                whisper_id,
                participant_ids: participants,
            }) => {
                assert_eq!(whisper_id, WhisperId::nil());
                assert_eq!(
                    participants,
                    BTreeSet::from([
                        ParticipantId::from_u128(0),
                        ParticipantId::from_u128(1),
                        ParticipantId::from_u128(2)
                    ])
                );
            }
            unexpected => panic!("Expected kick_whisper_participants message, got: {unexpected:?}"),
        }
    }

    #[test]
    fn accept_whisper_invite() {
        let json = json!({
            "action": "accept_whisper_invite",
            "whisper_id": "00000000-0000-0000-0000-000000000000"
        });

        match serde_json::from_value(json).unwrap() {
            SubroomAudioCommand::AcceptWhisperInvite { whisper_id } => {
                assert_eq!(whisper_id, WhisperId::nil());
            }
            unexpected => panic!("Expected accept_whisper_invite message, got: {unexpected:?}"),
        }
    }

    #[test]
    fn decline_whisper_invite() {
        let json = json!({
            "action": "decline_whisper_invite",
            "whisper_id": "00000000-0000-0000-0000-000000000000"
        });

        match serde_json::from_value(json).unwrap() {
            SubroomAudioCommand::DeclineWhisperInvite { whisper_id } => {
                assert_eq!(whisper_id, WhisperId::nil());
            }
            unexpected => panic!("Expected decline_whisper_invite message, got: {unexpected:?}"),
        }
    }

    #[test]
    fn leave_whisper_group() {
        let json = json!({
            "action": "leave_whisper_group",
            "whisper_id": "00000000-0000-0000-0000-000000000000"
        });

        match serde_json::from_value(json).unwrap() {
            SubroomAudioCommand::LeaveWhisperGroup { whisper_id } => {
                assert_eq!(whisper_id, WhisperId::nil());
            }
            unexpected => panic!("Expected leave_whisper_group message, got: {unexpected:?}"),
        }
    }
}

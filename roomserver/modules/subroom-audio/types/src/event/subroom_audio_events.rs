// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use super::{
    WhisperGroupOutgoing, error::SubroomAudioError,
    whisper_participant_info::WhisperParticipantInfo,
};
use crate::whisper_id::WhisperId;

/// Events sent out by the `subroom_audio` module
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "message")]
pub enum SubroomAudioEvent {
    /// The whisper group has been created
    WhisperGroupCreated {
        /// The livekit access token
        token: String,
        /// The whisper group
        #[serde(flatten)]
        group: WhisperGroupOutgoing,
    },
    /// An invite to a whisper group
    WhisperInvite {
        /// The issuer of the invite
        issuer: ParticipantId,
        /// The whisper group
        #[serde(flatten)]
        group: WhisperGroupOutgoing,
    },
    /// The access token for a whisper group
    WhisperToken {
        /// Unique id for the whisper group
        whisper_id: WhisperId,
        /// The JWT access token for the whisper room
        token: String,
    },
    /// Another participant was invited to the whisper group
    ParticipantsInvited {
        /// The id of the whisper group
        whisper_id: WhisperId,
        /// The participants that were invited
        participant_ids: Vec<ParticipantId>,
    },
    /// The invite to a whisper group was accepted
    WhisperInviteAccepted {
        /// The id of the whisper group
        whisper_id: WhisperId,
        /// The participant that accepted the invite
        participant_id: ParticipantId,
    },
    /// The participant declined the whisper invite
    WhisperInviteDeclined(WhisperParticipantInfo),
    /// Kicked from the whisper group
    Kicked {
        /// The id of the whisper group
        whisper_id: WhisperId,
    },
    /// A participant left the whisper group
    LeftWhisperGroup(WhisperParticipantInfo),
    /// An error occurred
    Error(SubroomAudioError),
}

impl From<SubroomAudioError> for SubroomAudioEvent {
    fn from(error: SubroomAudioError) -> Self {
        Self::Error(error)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use opentalk_types_signaling::ParticipantId;
    use serde_json::json;

    use super::SubroomAudioEvent;
    use crate::{
        event::{SubroomAudioError, WhisperParticipantInfo},
        state::{WhisperGroup, WhisperState},
        whisper_id::WhisperId,
    };

    #[test]
    fn serialize_whisper_group_created() {
        let group = WhisperGroup {
            whisper_id: WhisperId::nil(),
            participants: BTreeMap::from([
                (ParticipantId::from_u128(0), WhisperState::Creator),
                (ParticipantId::from_u128(1), WhisperState::Invited),
                (ParticipantId::from_u128(2), WhisperState::Accepted),
            ]),
        };

        let event = SubroomAudioEvent::WhisperGroupCreated {
            token: "<jwt-token>".into(),
            group: group.into(),
        };
        let value = serde_json::to_value(event).expect("Must be serializable");

        assert_eq!(
            value,
            json!({
                "message": "whisper_group_created",
                "whisper_id": "00000000-0000-0000-0000-000000000000",
                "token": "<jwt-token>",
                "participants": [
                    {
                        "participant_id": "00000000-0000-0000-0000-000000000000",
                        "state": "creator"
                    },
                    {
                        "participant_id": "00000000-0000-0000-0000-000000000001",
                        "state": "invited"
                    },
                    {
                        "participant_id": "00000000-0000-0000-0000-000000000002",
                        "state": "accepted"
                    }
                ],

            })
        );
    }

    #[test]
    fn serialize_whisper_invite() {
        let group = WhisperGroup {
            whisper_id: WhisperId::nil(),
            participants: BTreeMap::from([
                (ParticipantId::from_u128(0), WhisperState::Creator),
                (ParticipantId::from_u128(1), WhisperState::Invited),
                (ParticipantId::from_u128(2), WhisperState::Accepted),
            ]),
        };

        let event = SubroomAudioEvent::WhisperInvite {
            issuer: ParticipantId::from_u128(0),
            group: group.into(),
        };

        let value = serde_json::to_value(event).expect("Must be serializable");

        assert_eq!(
            value,
            json!({
                "message": "whisper_invite",
                "issuer": "00000000-0000-0000-0000-000000000000",
                "whisper_id": "00000000-0000-0000-0000-000000000000",
                "participants": [
                    {
                        "participant_id": "00000000-0000-0000-0000-000000000000",
                        "state": "creator"
                    },
                    {
                        "participant_id": "00000000-0000-0000-0000-000000000001",
                        "state": "invited"
                    },
                    {
                        "participant_id": "00000000-0000-0000-0000-000000000002",
                        "state": "accepted"
                    }
                ]
            })
        );
    }

    #[test]
    fn serialize_whisper_token() {
        let event = SubroomAudioEvent::WhisperToken {
            whisper_id: WhisperId::nil(),
            token: "<jwt-token>".into(),
        };

        let value = serde_json::to_value(event).expect("Must be serializable");
        assert_eq!(
            value,
            json!({
                "message": "whisper_token",
                "whisper_id": "00000000-0000-0000-0000-000000000000",
                "token": "<jwt-token>"
            })
        );
    }

    #[test]
    fn serialize_participants_invited() {
        let event = SubroomAudioEvent::ParticipantsInvited {
            whisper_id: WhisperId::nil(),
            participant_ids: vec![ParticipantId::from_u128(0), ParticipantId::from_u128(1)],
        };

        let value = serde_json::to_value(event).expect("Must be serializable");
        assert_eq!(
            value,
            json!({
                "message": "participants_invited",
                "whisper_id": "00000000-0000-0000-0000-000000000000",
                "participant_ids": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000001"]
            })
        );
    }

    #[test]
    fn serialize_whisper_invite_accepted() {
        let event = SubroomAudioEvent::WhisperInviteAccepted {
            whisper_id: WhisperId::nil(),
            participant_id: ParticipantId::nil(),
        };

        let value = serde_json::to_value(event).expect("Must be serializable");
        assert_eq!(
            value,
            json!({
                "message": "whisper_invite_accepted",
                "whisper_id": "00000000-0000-0000-0000-000000000000",
                "participant_id": "00000000-0000-0000-0000-000000000000"
            })
        );
    }

    #[test]
    fn serialize_whisper_invite_declined() {
        let event = SubroomAudioEvent::WhisperInviteDeclined(WhisperParticipantInfo {
            whisper_id: WhisperId::nil(),
            participant_id: ParticipantId::nil(),
        });

        let value = serde_json::to_value(event).expect("Must be serializable");
        assert_eq!(
            value,
            json!({
                "message": "whisper_invite_declined",
                "whisper_id": "00000000-0000-0000-0000-000000000000",
                "participant_id": "00000000-0000-0000-0000-000000000000"
            })
        );
    }

    #[test]
    fn serialize_kicked() {
        let event = SubroomAudioEvent::Kicked {
            whisper_id: WhisperId::nil(),
        };
        let value = serde_json::to_value(event).expect("Must be serializable");
        assert_eq!(
            value,
            json!({
                "whisper_id": "00000000-0000-0000-0000-000000000000",
                "message": "kicked",
            })
        );
    }

    #[test]
    fn serialize_left_whisper_group() {
        let event = SubroomAudioEvent::LeftWhisperGroup(WhisperParticipantInfo {
            whisper_id: WhisperId::nil(),
            participant_id: ParticipantId::nil(),
        });

        let value = serde_json::to_value(event).expect("Must be serializable");
        assert_eq!(
            value,
            json!({
                "message": "left_whisper_group",
                "whisper_id": "00000000-0000-0000-0000-000000000000",
                "participant_id": "00000000-0000-0000-0000-000000000000"
            })
        );
    }

    #[test]
    fn serialize_error_not_invited() {
        let event = SubroomAudioEvent::Error(SubroomAudioError::NotInvited);

        let value = serde_json::to_value(event).expect("Must be serializable");
        assert_eq!(
            value,
            json!({
                "message": "error",
                "error": "not_invited"
            })
        );
    }

    #[test]
    fn serialize_error_invalid_participant_targets() {
        let event = SubroomAudioEvent::Error(SubroomAudioError::InvalidParticipantTargets {
            participant_ids: vec![ParticipantId::nil()],
        });

        let value = serde_json::to_value(event).expect("Must be serializable");
        assert_eq!(
            value,
            json!({
                "message": "error",
                "error": "invalid_participant_targets",
                "participant_ids": ["00000000-0000-0000-0000-000000000000"]
            })
        );
    }
}

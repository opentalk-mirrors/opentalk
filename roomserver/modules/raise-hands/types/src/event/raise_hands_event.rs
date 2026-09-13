// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::event::RaiseHandsError;

/// Events sent out by the `moderation` module
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum RaiseHandsEvent {
    /// Sent out when raise hands is enabled
    RaiseHandsEnabled {
        /// The moderator who enabled raise hands
        issued_by: ParticipantId,
    },

    /// Sent out when raise hands is disabled
    RaiseHandsDisabled {
        /// The moderator who enabled raise hands
        issued_by: ParticipantId,
    },

    /// This participant raised a hand
    HandRaised {
        /// The participant that raised their hand
        participant: ParticipantId,
    },

    /// This participant lowered a hand
    HandLowered {
        /// The participant that lowered their hand
        participant: ParticipantId,
    },

    /// Sent out when raised hand is reset by a moderator
    RaisedHandResetByModerator {
        /// The moderator who reset raised hand
        issued_by: ParticipantId,
        /// The participants whose raised hands were reset
        participants: BTreeSet<ParticipantId>,
    },

    /// An error happened when executing a `moderation` command
    Error(RaiseHandsError),
}

impl From<RaiseHandsError> for RaiseHandsEvent {
    fn from(value: RaiseHandsError) -> Self {
        Self::Error(value)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_raise_hands_enabled() {
        let event = RaiseHandsEvent::RaiseHandsEnabled {
            issued_by: ParticipantId::nil(),
        };

        assert_snapshot!(serde_json::to_string_pretty(&event).unwrap(), @r#"
		{
		  "message": "raise_hands_enabled",
		  "issued_by": "00000000-0000-0000-0000-000000000000"
		}
		"#);
    }

    #[test]
    fn deserialize_raise_hands_enabled() {
        let json = json!({
            "message": "raise_hands_enabled",
            "issued_by": "00000000-0000-0000-0000-000000000000"
        });
        let produced: RaiseHandsEvent = serde_json::from_value(json).unwrap();
        let expected = RaiseHandsEvent::RaiseHandsEnabled {
            issued_by: ParticipantId::nil(),
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_raise_hands_disabled() {
        let event = RaiseHandsEvent::RaiseHandsDisabled {
            issued_by: ParticipantId::nil(),
        };

        assert_snapshot!(serde_json::to_string_pretty(&event).unwrap(), @r#"
		{
		  "message": "raise_hands_disabled",
		  "issued_by": "00000000-0000-0000-0000-000000000000"
		}
		"#);
    }

    #[test]
    fn deserialize_raise_hands_disabled() {
        let json = json!({
            "message": "raise_hands_disabled",
            "issued_by": "00000000-0000-0000-0000-000000000000"
        });
        let produced: RaiseHandsEvent = serde_json::from_value(json).unwrap();
        let expected = RaiseHandsEvent::RaiseHandsDisabled {
            issued_by: ParticipantId::nil(),
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_hand_raised() {
        let event = RaiseHandsEvent::HandRaised {
            participant: ParticipantId::nil(),
        };

        assert_snapshot!(serde_json::to_string_pretty(&event).unwrap(), @r#"
		{
		  "message": "hand_raised",
		  "participant": "00000000-0000-0000-0000-000000000000"
		}
		"#);
    }

    #[test]
    fn deserialize_hand_raised() {
        let json = json!({
            "message": "hand_raised",
            "participant": "00000000-0000-0000-0000-000000000000"
        });
        let produced: RaiseHandsEvent = serde_json::from_value(json).unwrap();
        let expected = RaiseHandsEvent::HandRaised {
            participant: ParticipantId::nil(),
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_hand_lowered() {
        let event = RaiseHandsEvent::HandLowered {
            participant: ParticipantId::nil(),
        };

        assert_snapshot!(serde_json::to_string_pretty(&event).unwrap(), @r#"
		{
		  "message": "hand_lowered",
		  "participant": "00000000-0000-0000-0000-000000000000"
		}
		"#);
    }

    #[test]
    fn deserialize_hand_lowered() {
        let json = json!({
            "message": "hand_lowered",
            "participant": "00000000-0000-0000-0000-000000000000"
        });
        let produced: RaiseHandsEvent = serde_json::from_value(json).unwrap();
        let expected = RaiseHandsEvent::HandLowered {
            participant: ParticipantId::nil(),
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_raised_hand_reset_by_moderator() {
        let event = RaiseHandsEvent::RaisedHandResetByModerator {
            issued_by: ParticipantId::nil(),
            participants: BTreeSet::from_iter([ParticipantId::nil()]),
        };

        assert_snapshot!(serde_json::to_string_pretty(&event).unwrap(), @r#"
		{
		  "message": "raised_hand_reset_by_moderator",
		  "issued_by": "00000000-0000-0000-0000-000000000000",
		  "participants": [
		    "00000000-0000-0000-0000-000000000000"
		  ]
		}
		"#);
    }

    #[test]
    fn deserialize_raised_hand_reset_by_moderator() {
        let json = json!({
            "message": "raised_hand_reset_by_moderator",
            "issued_by": "00000000-0000-0000-0000-000000000000",
            "participants": ["00000000-0000-0000-0000-000000000000"]
        });
        let produced: RaiseHandsEvent = serde_json::from_value(json).unwrap();
        let mut participants = BTreeSet::new();
        participants.insert(ParticipantId::nil());
        let expected = RaiseHandsEvent::RaisedHandResetByModerator {
            issued_by: ParticipantId::nil(),
            participants,
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_error() {
        let event = RaiseHandsEvent::Error(RaiseHandsError::RaiseHandsDisabled);

        assert_snapshot!(serde_json::to_string_pretty(&event).unwrap(), @r#"
        {
          "message": "error",
          "error": "raise_hands_disabled"
        }
        "#);
    }

    #[test]
    fn deserialize_error() {
        let json = json!({
            "message": "error",
            "error": "raise_hands_disabled"
        });

        let produced: RaiseHandsEvent = serde_json::from_value(json).unwrap();

        assert_eq!(
            produced,
            RaiseHandsEvent::Error(RaiseHandsError::RaiseHandsDisabled)
        );
    }
}

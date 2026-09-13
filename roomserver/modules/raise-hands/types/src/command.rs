// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `moderation` namespace

use std::collections::BTreeSet;

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::event::RaiseHandsEvent;

/// Commands for the `moderation` namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum RaiseHandsCommand {
    /// Enable raise hands for the meeting
    EnableRaiseHands,

    /// Disable raise hands for the meeting
    DisableRaiseHands,

    /// Raise a hand
    RaiseHand,

    /// Lower a raised hand
    LowerHand,

    /// Reset raised hands for the meeting
    ResetRaisedHands {
        /// An optional single participant to reset the raised hand for
        #[serde(
            default,
            with = "opentalk_types_common::collections::one_or_many_btree_set_option"
        )]
        target: Option<BTreeSet<ParticipantId>>,
    },
}

impl CreateReplica<RaiseHandsEvent> for RaiseHandsCommand {
    fn replicate(&self) -> Option<RaiseHandsEvent> {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use insta::assert_snapshot;
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_enable_raise_hands() {
        let cmd: RaiseHandsCommand = RaiseHandsCommand::EnableRaiseHands;

        assert_snapshot!(serde_json::to_string_pretty(&cmd).unwrap(), @r#"
        {
          "action": "enable_raise_hands"
        }
        "#);
    }

    #[test]
    fn deserialize_enable_raise_hands() {
        let json = json!({
            "action": "enable_raise_hands"
        });
        let produced: RaiseHandsCommand = serde_json::from_value(json).unwrap();
        let expected = RaiseHandsCommand::EnableRaiseHands;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_disable_raise_hands() {
        let cmd: RaiseHandsCommand = RaiseHandsCommand::DisableRaiseHands;

        assert_snapshot!(serde_json::to_string_pretty(&cmd).unwrap(), @r#"
        {
          "action": "disable_raise_hands"
        }
        "#);
    }

    #[test]
    fn deserialize_disable_raise_hands() {
        let json = json!({
            "action": "disable_raise_hands"
        });
        let produced: RaiseHandsCommand = serde_json::from_value(json).unwrap();
        let expected = RaiseHandsCommand::DisableRaiseHands;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_raise_hand() {
        let cmd: RaiseHandsCommand = RaiseHandsCommand::RaiseHand;

        assert_snapshot!(serde_json::to_string_pretty(&cmd).unwrap(), @r#"
        {
          "action": "raise_hand"
        }
        "#);
    }

    #[test]
    fn deserialize_raise_hand() {
        let json = json!({
            "action": "raise_hand"
        });
        let produced: RaiseHandsCommand = serde_json::from_value(json).unwrap();
        let expected = RaiseHandsCommand::RaiseHand;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_lower_hand() {
        let cmd: RaiseHandsCommand = RaiseHandsCommand::LowerHand;

        assert_snapshot!(serde_json::to_string_pretty(&cmd).unwrap(), @r#"
        {
          "action": "lower_hand"
        }
        "#);
    }

    #[test]
    fn deserialize_lower_hand() {
        let json = json!({
            "action": "lower_hand"
        });
        let produced: RaiseHandsCommand = serde_json::from_value(json).unwrap();
        let expected = RaiseHandsCommand::LowerHand;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_reset_raise_hands_for_single_participant() {
        let cmd = RaiseHandsCommand::ResetRaisedHands {
            target: Some(BTreeSet::from_iter([ParticipantId::nil()])),
        };

        assert_snapshot!(serde_json::to_string_pretty(&cmd).unwrap(), @r#"
        {
          "action": "reset_raised_hands",
          "target": [
            "00000000-0000-0000-0000-000000000000"
          ]
        }
        "#);
    }

    #[test]
    fn deserialize_reset_raised_hand_for_single_participant() {
        let json = json!({
            "action": "reset_raised_hands",
            "target": "00000000-0000-0000-0000-000000000000"
        });

        let msg: RaiseHandsCommand = serde_json::from_value(json).unwrap();

        if let RaiseHandsCommand::ResetRaisedHands { target } = msg {
            assert_eq!(target, Some(BTreeSet::from_iter([ParticipantId::nil()])));
        } else {
            panic!()
        }
    }

    #[test]
    fn serialize_reset_raise_hands_for_multiple_participants() {
        let cmd = RaiseHandsCommand::ResetRaisedHands {
            target: Some(BTreeSet::from_iter([
                ParticipantId::nil(),
                ParticipantId::from_u128(0xcafe),
            ])),
        };

        assert_snapshot!(serde_json::to_string_pretty(&cmd).unwrap(), @r#"
        {
          "action": "reset_raised_hands",
          "target": [
            "00000000-0000-0000-0000-000000000000",
            "00000000-0000-0000-0000-00000000cafe"
          ]
        }
        "#);
    }

    #[test]
    fn deserialize_reset_raised_hand_for_multiple_participants() {
        let json = json!({
            "action": "reset_raised_hands",
            "target": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-00000000cafe"]
        });

        let cmd: RaiseHandsCommand = serde_json::from_value(json).unwrap();

        if let RaiseHandsCommand::ResetRaisedHands { target } = cmd {
            assert_eq!(
                target,
                Some(BTreeSet::from_iter([
                    ParticipantId::nil(),
                    ParticipantId::from_u128(0xcafe)
                ]))
            );
        } else {
            panic!()
        }
    }

    #[test]
    fn serialize_reset_raise_hands_for_all_participants() {
        let cmd = RaiseHandsCommand::ResetRaisedHands { target: None };

        assert_snapshot!(serde_json::to_string_pretty(&cmd).unwrap(), @r#"
        {
          "action": "reset_raised_hands",
          "target": null
        }
        "#);
    }

    #[test]
    fn deserialize_reset_raised_hands_for_all_participants() {
        let json = json!({
            "action": "reset_raised_hands"
        });

        let msg: RaiseHandsCommand = serde_json::from_value(json).unwrap();

        if let RaiseHandsCommand::ResetRaisedHands { target } = msg {
            assert!(target.is_none());
        } else {
            panic!()
        }
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling cancel reason for the `legal-vote` namespace.

use serde::{Deserialize, Serialize};

use crate::cancel::CustomCancelReason;

/// The reason for a cancel.
#[derive(Debug, Clone, Eq, PartialOrd, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "reason", content = "custom")]
pub enum CancelReason {
    /// The room got destroyed and the server canceled the vote.
    RoomDestroyed,

    /// The initiator left the room and the server canceled the vote.
    InitiatorLeft,

    /// Custom reason for a cancel.
    Custom(CustomCancelReason),
}

impl From<CustomCancelReason> for CancelReason {
    fn from(value: CustomCancelReason) -> Self {
        Self::Custom(value)
    }
}

#[cfg(test)]
mod test {
    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_room_destroyed_cancel_reason() {
        let produced = serde_json::to_string_pretty(&CancelReason::RoomDestroyed).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "reason": "room_destroyed"
        }
        "#);
    }

    #[test]
    fn deserialize_room_destroyed_cancel_reason() {
        let produced: CancelReason = serde_json::from_value(json!({
            "reason": "room_destroyed",
        }))
        .unwrap();

        let expected = CancelReason::RoomDestroyed;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_initiator_left_cancel_reason() {
        let produced = serde_json::to_string_pretty(&CancelReason::InitiatorLeft).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "reason": "initiator_left"
        }
        "#);
    }

    #[test]
    fn deserialize_initiator_left_cancel_reason() {
        let produced: CancelReason = serde_json::from_value(json!({
            "reason": "initiator_left",
        }))
        .unwrap();

        let expected = CancelReason::InitiatorLeft;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_custom_cancel_reason() {
        let produced = serde_json::to_string_pretty(&CancelReason::Custom(
            CustomCancelReason::try_from("Test Reason").unwrap(),
        ))
        .unwrap();

        assert_snapshot!(produced, @r#"
        {
          "reason": "custom",
          "custom": "Test Reason"
        }
        "#);
    }

    #[test]
    fn deserialize_custom_cancel_reason() {
        let produced: CancelReason = serde_json::from_value(json!({
            "reason": "custom",
            "custom": "Test Reason",
        }))
        .unwrap();

        let expected = CancelReason::Custom(CustomCancelReason::try_from("Test Reason").unwrap());

        assert_eq!(produced, expected);
    }
}

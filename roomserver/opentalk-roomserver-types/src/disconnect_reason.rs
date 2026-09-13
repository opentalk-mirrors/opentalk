// SPDX-License-Identifier: EUPL-1.2
//
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisconnectReason {
    /// The participant left the conference
    Leave,
    /// The connection was interrupted
    ConnectionLost,
    /// The participant was kicked by a moderator
    Kicked,
    /// The participant was removed from the conference because debriefing was started
    Debriefed,
    /// The participant was banned from the room
    Banned,
    /// The participant was sent to the waiting room
    SentToWaitingRoom,
    /// The participant is a guest and guest access has been disabled
    GuestAccessDisabled,
    /// The participant was removed due to an internal error
    InternalError,
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_leave() {
        let reason = DisconnectReason::Leave;
        let produced = serde_json::to_string_pretty(&reason).unwrap();

        assert_snapshot!(produced, @r#""leave""#);
    }

    #[test]
    fn deserialize_leave() {
        let produced: DisconnectReason = serde_json::from_value(json!("leave")).unwrap();
        let expected = DisconnectReason::Leave;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_connection_lost() {
        let reason = DisconnectReason::ConnectionLost;
        let produced = serde_json::to_string_pretty(&reason).unwrap();

        assert_snapshot!(produced, @r#""connection_lost""#);
    }

    #[test]
    fn deserialize_connection_lost() {
        let produced: DisconnectReason = serde_json::from_value(json!("connection_lost")).unwrap();
        let expected = DisconnectReason::ConnectionLost;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_kicked() {
        let reason = DisconnectReason::Kicked;
        let produced = serde_json::to_string_pretty(&reason).unwrap();

        assert_snapshot!(produced, @r#""kicked""#);
    }

    #[test]
    fn deserialize_kicked() {
        let produced: DisconnectReason = serde_json::from_value(json!("kicked")).unwrap();
        let expected = DisconnectReason::Kicked;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_debriefed() {
        let reason = DisconnectReason::Debriefed;
        let produced = serde_json::to_string_pretty(&reason).unwrap();

        assert_snapshot!(produced, @r#""debriefed""#);
    }

    #[test]
    fn deserialize_debriefed() {
        let produced: DisconnectReason = serde_json::from_value(json!("debriefed")).unwrap();
        let expected = DisconnectReason::Debriefed;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_banned() {
        let reason = DisconnectReason::Banned;
        let produced = serde_json::to_string_pretty(&reason).unwrap();

        assert_snapshot!(produced, @r#""banned""#);
    }

    #[test]
    fn deserialize_banned() {
        let produced: DisconnectReason = serde_json::from_value(json!("banned")).unwrap();
        let expected = DisconnectReason::Banned;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_sent_to_waiting_room() {
        let reason = DisconnectReason::SentToWaitingRoom;
        let produced = serde_json::to_string_pretty(&reason).unwrap();

        assert_snapshot!(produced, @r#""sent_to_waiting_room""#);
    }

    #[test]
    fn deserialize_sent_to_waiting_room() {
        let produced: DisconnectReason =
            serde_json::from_value(json!("sent_to_waiting_room")).unwrap();
        let expected = DisconnectReason::SentToWaitingRoom;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_guest_access_disabled() {
        let reason = DisconnectReason::GuestAccessDisabled;
        let produced = serde_json::to_string_pretty(&reason).unwrap();

        assert_snapshot!(produced, @r#""guest_access_disabled""#);
    }

    #[test]
    fn deserialize_guest_access_disabled() {
        let produced: DisconnectReason =
            serde_json::from_value(json!("guest_access_disabled")).unwrap();
        let expected = DisconnectReason::GuestAccessDisabled;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_internal_error() {
        let reason = DisconnectReason::InternalError;
        let produced = serde_json::to_string_pretty(&reason).unwrap();

        assert_snapshot!(produced, @r#""internal_error""#);
    }

    #[test]
    fn deserialize_internal_error() {
        let produced: DisconnectReason = serde_json::from_value(json!("internal_error")).unwrap();
        let expected = DisconnectReason::InternalError;

        assert_eq!(produced, expected);
    }
}

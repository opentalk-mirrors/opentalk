// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use opentalk_roomserver_types::connection_id::ConnectionId;
use serde::{Deserialize, Serialize};

use crate::{E2eeEvent, MlsMessages, WelcomeMessage};

#[derive(Deserialize, Serialize, PartialEq, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum E2eeCommand {
    Invite {
        invitee: ConnectionId,
        welcome_message: WelcomeMessage,
        /// Proposal and commit sent to the existing members of the group
        mls_messages: MlsMessages,
    },
    Message(MlsMessages),
}

impl CreateReplica<E2eeEvent> for E2eeCommand {
    fn replicate(&self) -> Option<E2eeEvent> {
        // Replication is handled by the module.
        None
    }
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use bytes::Bytes;
    use opentalk_roomserver_types::connection_id::ConnectionId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::WelcomeMessage;

    const SAMPLE_UUID: &str = "6650b3e2-5f1c-4073-951e-cc4bcd6ddfef";

    fn sample_invite_command() -> E2eeCommand {
        E2eeCommand::Invite {
            invitee: ConnectionId::from_str(SAMPLE_UUID).expect("SAMPLE_UUID must be a valid UUID"),
            welcome_message: WelcomeMessage {
                welcome: Bytes::from_static(b"welcome-bytes"),
                ratchet_tree: Bytes::from_static(b"ratchet-tree-bytes"),
            },
            mls_messages: MlsMessages {
                payload: vec![Bytes::from_static(b"mls1"), Bytes::from_static(b"mls2")],
            },
        }
    }

    fn sample_mls_messages() -> MlsMessages {
        MlsMessages {
            payload: vec![Bytes::from_static(b"mls1"), Bytes::from_static(b"mls2")],
        }
    }

    #[test]
    fn serialize_invite_command() {
        let cmd = sample_invite_command();
        let serialized = serde_json::to_value(&cmd).unwrap();

        let expected = json!({
            "action": "invite",
            "invitee": SAMPLE_UUID,
            "welcome_message": {
                "welcome": b"welcome-bytes",
                "ratchet_tree": b"ratchet-tree-bytes",
            },
            "mls_messages": {
                "payload": [
                    b"mls1",
                    b"mls2",
                ]
            }
        });

        assert_eq!(
            serialized, expected,
            "Invite serialization mismatch.\nExpected: {expected:?}\nGot: {serialized:?}"
        );
    }

    #[test]
    fn serialize_message_command() {
        let mls_messages = sample_mls_messages();
        let cmd = E2eeCommand::Message(mls_messages.clone());
        let serialized = serde_json::to_value(&cmd).unwrap();

        let expected = json!({
            "action": "message",
            "payload": [
                b"mls1",
                b"mls2",
            ]
        });

        assert_eq!(
            serialized, expected,
            "Message serialization mismatch.\nExpected: {expected:?}\nGot: {serialized:?}"
        );
    }

    #[test]
    fn deserialize_invite_command() {
        let expected = sample_invite_command();
        let value = json!({
            "action": "invite",
            "invitee": SAMPLE_UUID,
            "welcome_message": {
                "welcome": b"welcome-bytes",
                "ratchet_tree": b"ratchet-tree-bytes",
            },
            "mls_messages": {
                "payload": [
                    b"mls1",
                    b"mls2",
                ]
            }
        });

        let cmd: E2eeCommand = serde_json::from_value(value).unwrap();
        assert_eq!(cmd, expected);
    }

    #[test]
    fn deserialize_message_command() {
        let mls_messages = sample_mls_messages();
        let value = json!({
            "action": "message",
            "payload": [
                b"mls1",
                b"mls2",
            ]
        });

        let cmd: E2eeCommand = serde_json::from_value(value).unwrap();
        assert_eq!(cmd, E2eeCommand::Message(mls_messages.clone()),);
    }

    #[test]
    fn replicate_message() {
        let mls_messages = sample_mls_messages();
        let cmd = E2eeCommand::Message(mls_messages.clone());
        let event = cmd.replicate();
        assert_eq!(event, None);
    }
}

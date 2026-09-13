// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling state for the `chat` namespace

use std::collections::BTreeMap;

use opentalk_types_common::{time::Timestamp, users::GroupName};
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::state::{ChatChunk, PrivateHistory};

/// The state of the `chat` module
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatState {
    /// Is the chat module enabled
    pub enabled: bool,

    /// Chat history for the whole conference room
    ///
    /// Can still be accessed when the participant is in a breakout room
    pub global_history: ChatChunk,

    /// Chat history for the current breakout room
    ///
    /// Only present when the associated participant is in a breakout room
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakout_room_history: Option<ChatChunk>,

    /// All private chat history in the room
    pub private_history: Vec<PrivateHistory>,

    /// Timestamp for last time someone read a message
    pub last_seen_timestamp_global: Option<Timestamp>,

    /// Last seen timestamp of the current breakout room
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen_timestamp_breakout: Option<Timestamp>,

    /// Timestamp for last time someone read a private message
    pub last_seen_timestamps_private: BTreeMap<ParticipantId, Timestamp>,

    /// Timestamp for last time someone read a group message
    pub last_seen_timestamps_group: BTreeMap<GroupName, Timestamp>,
}

impl opentalk_types_signaling::SignalingModuleFrontendData for ChatState {
    const NAMESPACE: Option<opentalk_types_common::modules::ModuleId> = Some(crate::CHAT_MODULE_ID);
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::{MessageId, Scope, state::StoredMessage};

    #[test]
    fn server_message() {
        let expected = json!({
            "id":"00000000-0000-0000-0000-000000000000",
            "source":"00000000-0000-0000-0000-000000000000",
            "timestamp":"1970-01-01T00:00:00Z",
            "content":"Hello All!",
            "scope":"global",
        });

        let produced = serde_json::to_value(StoredMessage {
            id: MessageId::nil(),
            source: ParticipantId::nil(),
            timestamp: Timestamp::unix_epoch(),
            content: "Hello All!".to_string(),
            scope: Scope::Global,
        })
        .unwrap();

        assert_eq!(expected, produced);
    }
}

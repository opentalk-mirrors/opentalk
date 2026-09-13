// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `chat` namespace

use opentalk_types_common::time::Timestamp;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::{
    MessageId, Scope,
    event::ChatError,
    state::{BreakoutHistory, ChatChunk, PrivateHistory},
};

/// A chat event which occurred
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum ChatEvent {
    /// Chat event where chat was enabled
    ChatEnabled {
        /// Participant who enabled the chat
        issued_by: ParticipantId,
    },

    /// Chat event where chat was disabled
    ChatDisabled {
        /// Participant who disabled the chat
        issued_by: ParticipantId,
    },

    /// A chunk of the rooms chat history
    RoomChatHistoryChunk {
        /// Room chat history chunk
        history: ChatChunk,
    },

    /// A chunk of a breakout rooms chat history
    BreakoutChatHistoryChunk(BreakoutHistory),

    /// A chunk of a private chat history between two participants
    PrivateChatHistoryChunk(PrivateHistory),

    /// Chat event where a message was sent
    MessageSent {
        /// Id of the message
        id: MessageId,

        /// Sender of the message
        source: ParticipantId,

        /// Content of the message
        content: String,

        /// Scope of the message
        #[serde(flatten)]
        scope: Scope,
    },

    /// Chat event where history was cleared
    HistoryCleared {
        /// ID of the participant that cleared chat history
        issued_by: ParticipantId,
    },

    /// Chat event when last seen timestamp was set.
    SetLastSeenTimestamp {
        /// Scope of the timestamp
        #[serde(flatten)]
        scope: Scope,

        /// Last seen timestamp
        timestamp: Timestamp,
    },

    /// The results of a search
    SearchResults {
        /// A chunk of messages matching the search term
        matches: ChatChunk,
        /// The [`Scope`] of the messages
        #[serde(flatten)]
        scope: Scope,
    },

    /// The client is sending too many messages and should slow down
    ///
    /// When the client does not slow down, further messages may be rejected with a
    /// [`ChatError::TooManyRequests`] error.
    SlowDown {
        /// The time the client should wait before sending another chat message
        recommended_wait_ms: u32,
    },

    /// A participant has been rate limited
    ParticipantRateLimited { participant_id: ParticipantId },

    /// Chat event which errored, see [`ChatError`]
    Error(ChatError),
}

impl From<ChatError> for ChatEvent {
    fn from(value: ChatError) -> Self {
        Self::Error(value)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_types_common::time::Timestamp;
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::{MessageId, Scope};

    #[test]
    fn serialize_chat_enabled() {
        let event = ChatEvent::ChatEnabled {
            issued_by: ParticipantId::from_u128(1),
        };
        let serialized = serde_json::to_value(&event).expect("Serialization failed");
        assert_eq!(
            serialized,
            json!({
                "message":"chat_enabled",
                "issued_by":"00000000-0000-0000-0000-000000000001"
            })
        );
    }

    #[test]
    fn deserialize_chat_enabled() {
        let json_data = json!(
            {
                "message":"chat_enabled",
                "issued_by":"00000000-0000-0000-0000-000000000001"
            }
        );
        let deserialized: ChatEvent =
            serde_json::from_value(json_data).expect("Deserialization failed");
        if let ChatEvent::ChatEnabled { issued_by } = deserialized {
            assert_eq!(issued_by, ParticipantId::from_u128(1));
            if let ChatEvent::ChatEnabled { issued_by } = deserialized {
                assert_eq!(issued_by, ParticipantId::from_u128(1));
            } else {
                panic!("Expected ChatEvent::ChatEnabled");
            }
        }
    }

    #[test]
    fn serialize_chat_disabled() {
        let event = ChatEvent::ChatDisabled {
            issued_by: ParticipantId::from_u128(2),
        };
        let serialized = serde_json::to_value(&event).expect("Serialization failed");
        assert_eq!(
            serialized,
            json!(
                {
                    "message":"chat_disabled",
                    "issued_by":"00000000-0000-0000-0000-000000000002"
                }
            )
        );
    }

    #[test]
    fn deserialize_chat_disabled() {
        let json_data = json!(
            {
                "message":"chat_disabled",
                "issued_by":"00000000-0000-0000-0000-000000000002"
            }
        );

        let deserialized: ChatEvent =
            serde_json::from_value(json_data).expect("Deserialization failed");
        if let ChatEvent::ChatDisabled { issued_by } = deserialized {
            assert_eq!(issued_by, ParticipantId::from_u128(2));
            if let ChatEvent::ChatDisabled { issued_by } = deserialized {
                assert_eq!(issued_by, ParticipantId::from_u128(2));
            } else {
                panic!("Expected ChatEvent::ChatDisabled");
            }
        }
    }

    #[test]
    fn serialize_history_cleared() {
        let event = ChatEvent::HistoryCleared {
            issued_by: ParticipantId::from_u128(4),
        };
        let serialized = serde_json::to_value(&event).expect("Serialization failed");
        assert_eq!(
            serialized,
            json!({
                "message":"history_cleared",
                "issued_by":"00000000-0000-0000-0000-000000000004"
            })
        );
    }

    #[test]
    fn deserialize_history_cleared() {
        let json_data = json!(
            {
                "message":"history_cleared",
                "issued_by":"00000000-0000-0000-0000-000000000004"
            }
        );

        let deserialized: ChatEvent =
            serde_json::from_value(json_data).expect("Deserialization failed");
        if let ChatEvent::HistoryCleared { issued_by } = deserialized {
            assert_eq!(issued_by, ParticipantId::from_u128(4));
        } else {
            panic!("Expected ChatEvent::HistoryCleared");
        }
    }

    #[test]
    fn serialize_set_last_seen_timestamp() {
        let event = ChatEvent::SetLastSeenTimestamp {
            scope: Scope::Global,
            timestamp: Timestamp::unix_epoch(),
        };
        let serialized = serde_json::to_value(&event).expect("Serialization failed");
        assert_eq!(
            serialized,
            json!({
                "message":"set_last_seen_timestamp",
                "scope":"global",
                "timestamp":"1970-01-01T00:00:00Z"
            })
        );
    }

    #[test]
    fn deserialize_set_last_seen_timestamp() {
        let json_data = json!(
            {
                "message":"set_last_seen_timestamp",
                "scope":"global",
                "timestamp": "1970-01-01T00:00:00Z"
            }
        );

        let deserialized: ChatEvent =
            serde_json::from_value(json_data).expect("Deserialization failed");
        if let ChatEvent::SetLastSeenTimestamp { scope, .. } = deserialized {
            assert_eq!(scope, Scope::Global);
        } else {
            panic!("Expected ChatEvent::HistoryCleared");
        }
    }

    #[test]
    fn serialize_global_message() {
        let produced = serde_json::to_value(ChatEvent::MessageSent {
            id: MessageId::nil(),
            source: ParticipantId::nil(),
            content: "Hello All!".to_string(),
            scope: Scope::Global,
        })
        .unwrap();

        let expected = json!({
            "message": "message_sent",
            "id": "00000000-0000-0000-0000-000000000000",
            "source": "00000000-0000-0000-0000-000000000000",
            "content": "Hello All!",
            "scope": "global"
        });

        assert_eq!(expected, produced);
    }

    #[test]
    fn deserialize_global_message() {
        let json_data = json!(
            {
                "message":"message_sent",
                "id":"00000000-0000-0000-0000-00000000007b",
                "source":"00000000-0000-0000-0000-000000000003",
                "content":"Hello, world!",
                "scope":"global"
            }
        );

        let deserialized: ChatEvent =
            serde_json::from_value(json_data).expect("Deserialization failed");
        if let ChatEvent::MessageSent {
            id,
            source,
            content,
            scope,
        } = deserialized
        {
            assert_eq!(id, MessageId::from_u128(123));
            assert_eq!(source, ParticipantId::from_u128(3));
            assert_eq!(content, "Hello, world!");
            assert_eq!(scope, Scope::Global);
        } else {
            panic!("Expected ChatEvent::MessageSent");
        }
    }

    #[test]
    fn serialize_private_message() {
        let produced = serde_json::to_value(ChatEvent::MessageSent {
            id: MessageId::nil(),
            source: ParticipantId::nil(),
            content: "Hello All!".to_string(),
            scope: Scope::Private(ParticipantId::from_u128(1)),
        })
        .unwrap();

        let expected = json!({
            "message": "message_sent",
            "id": "00000000-0000-0000-0000-000000000000",
            "source": "00000000-0000-0000-0000-000000000000",
            "content": "Hello All!",
            "scope": "private",
            "target": "00000000-0000-0000-0000-000000000001",
        });
        assert_eq!(expected, produced);
    }

    #[test]
    fn deserialize_private_message() {
        let json_data = json!(
            {
                "message":"message_sent",
                "id":"00000000-0000-0000-0000-00000000007b",
                "source":"00000000-0000-0000-0000-000000000003",
                "content":"Hello, world!",
                "scope":"private",
                "target":"00000000-0000-0000-0000-000000000001"
            }
        );

        let deserialized: ChatEvent =
            serde_json::from_value(json_data).expect("Deserialization failed");
        if let ChatEvent::MessageSent {
            id,
            source,
            content,
            scope,
        } = deserialized
        {
            assert_eq!(id, MessageId::from_u128(123));
            assert_eq!(source, ParticipantId::from_u128(3));
            assert_eq!(content, "Hello, world!");
            assert_eq!(scope, Scope::Private(ParticipantId::from_u128(1)));
        } else {
            panic!("Expected ChatEvent::MessageSent");
        }
    }

    #[test]
    fn serialize_error() {
        let produced = serde_json::to_value(ChatEvent::Error(ChatError::ChatDisabled)).unwrap();

        let expected = json!({
            "message": "error",
            "error": "chat_disabled",
        });
        assert_eq!(expected, produced);
    }

    #[test]
    fn deserialize_error() {
        let json_data = json!(
            {
                "message":"error",
                "error":"insufficient_permissions"
            }
        );

        let deserialized: ChatEvent =
            serde_json::from_value(json_data).expect("Deserialization failed");
        if let ChatEvent::Error(ChatError::InsufficientPermissions) = deserialized {
            // Success
        } else {
            panic!("Expected ChatEvent::Error(ChatError::InsufficientPermissions)");
        }
    }

    #[test]
    fn serialize_too_many_requests() {
        let produced = serde_json::to_string_pretty(&ChatError::TooManyRequests {
            retry_after_ms: 500,
        })
        .expect("Failed to serialize");

        assert_snapshot!(produced, @r#"
        {
          "error": "too_many_requests",
          "retry_after_ms": 500
        }
        "#);
    }

    #[test]
    fn deserialize_too_many_requests() {
        let produced: ChatError = serde_json::from_value(json!({
            "error": "too_many_requests",
            "retry_after_ms": 500,
        }))
        .expect("Failed to serialize");

        let expected = ChatError::TooManyRequests {
            retry_after_ms: 500,
        };
        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_room_chat_history_chunk() {
        let produced = serde_json::to_string_pretty(&ChatEvent::RoomChatHistoryChunk {
            history: ChatChunk::default(),
        })
        .expect("Serialization failed");

        assert_snapshot!(produced, @r#"
        {
          "message": "room_chat_history_chunk",
          "history": {
            "messages": [],
            "next_index": null
          }
        }
        "#);
    }

    #[test]
    fn deserialize_room_chat_history_chunk() {
        let json = json!({
          "message": "room_chat_history_chunk",
          "history": {
            "messages": [],
            "next_index": null
          }
        });
        let produced = serde_json::from_value(json).expect("Deserialization failed");

        let expected = ChatEvent::RoomChatHistoryChunk {
            history: ChatChunk::default(),
        };

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_private_chat_history_chunk() {
        let produced =
            serde_json::to_string_pretty(&ChatEvent::PrivateChatHistoryChunk(PrivateHistory {
                correspondent: ParticipantId::nil(),
                history: ChatChunk::default(),
            }))
            .expect("Serialization failed");

        assert_snapshot!(produced, @r#"
        {
          "message": "private_chat_history_chunk",
          "correspondent": "00000000-0000-0000-0000-000000000000",
          "history": {
            "messages": [],
            "next_index": null
          }
        }
        "#);
    }

    #[test]
    fn deserialize_private_chat_history_chunk() {
        let json = json!(        {
          "message": "private_chat_history_chunk",
          "correspondent": "00000000-0000-0000-0000-000000000000",
          "history": {
            "messages": [],
            "next_index": null
          }
        });
        let produced = serde_json::from_value(json).expect("Deserialization failed");

        let expected = ChatEvent::PrivateChatHistoryChunk(PrivateHistory {
            correspondent: ParticipantId::nil(),
            history: ChatChunk::default(),
        });

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_search_results() {
        let produced = serde_json::to_string_pretty(&ChatEvent::SearchResults {
            matches: ChatChunk::default(),
            scope: Scope::Global,
        })
        .expect("Serialization failed");

        assert_snapshot!(produced, @r#"
        {
          "message": "search_results",
          "matches": {
            "messages": [],
            "next_index": null
          },
          "scope": "global"
        }
        "#);
    }

    #[test]
    fn deserialize_search_results() {
        let json = json!({
          "message": "search_results",
          "matches": {
            "messages": [],
            "next_index": null
          },
          "scope": "global"
        });
        let produced = serde_json::from_value(json).expect("Deserialization failed");

        let expected = ChatEvent::SearchResults {
            matches: ChatChunk::default(),
            scope: Scope::Global,
        };

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_slow_down() {
        let produced = serde_json::to_string_pretty(&ChatEvent::SlowDown {
            recommended_wait_ms: 500,
        })
        .expect("Serialization failed");

        assert_snapshot!(produced, @r#"
        {
          "message": "slow_down",
          "recommended_wait_ms": 500
        }
        "#);
    }

    #[test]
    fn deserialize_slow_down() {
        let produced: ChatEvent = serde_json::from_value(json!({
            "message": "slow_down",
            "recommended_wait_ms": 500,
        }))
        .expect("Deserialization failed");
        let expected = ChatEvent::SlowDown {
            recommended_wait_ms: 500,
        };

        assert_eq!(produced, expected);
    }
}

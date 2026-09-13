// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Chat history is retrieved in chunks. This is to avoid sending a giant list
//! of chat messages when a participant joins.

use serde::{Deserialize, Serialize};

use crate::state::StoredMessage;

/// The maximum number of messages that a [`ChatChunk`] can contain
pub const CHAT_CHUNK_SIZE: u32 = 100;

/// A chunk of the chat message history
///
/// The specific messages in a chunk will depend on the clients request. The first
/// [`ChatChunk`] is received when joining the room, containing the most recent
/// messages and the index to the next chunk. Messages are ordered chronologically,
/// the next chunk always contains older messages than the current one.
///
/// Further requests to fetch message history chunks need to contain the index
/// received with the previous chunk. A missing index indicates that no older
/// messages exist.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatChunk {
    /// The messages in this chunk
    pub messages: Vec<StoredMessage>,
    /// The message index of the newest message of the next chunk. Must be provided
    /// when requesting the next chunk.
    pub next_index: Option<u32>,
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_types_common::time::Timestamp;
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::{
        MessageId, Scope,
        state::{ChatChunk, StoredMessage},
    };

    #[test]
    fn serialize_chat_chunk() {
        let chunk = ChatChunk {
            messages: vec![StoredMessage {
                id: MessageId::nil(),
                source: ParticipantId::nil(),
                timestamp: Timestamp::unix_epoch(),
                content: "hello".into(),
                scope: Scope::Global,
            }],
            next_index: None,
        };
        let produced = serde_json::to_string_pretty(&chunk).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "messages": [
            {
              "id": "00000000-0000-0000-0000-000000000000",
              "source": "00000000-0000-0000-0000-000000000000",
              "timestamp": "1970-01-01T00:00:00Z",
              "content": "hello",
              "scope": "global"
            }
          ],
          "next_index": null
        }
        "#);
    }

    #[test]
    fn deserialize_chat_chunk() {
        let json = json!({
            "messages": [{
                "id": "00000000-0000-0000-0000-000000000000",
                "source": "00000000-0000-0000-0000-000000000000",
                "timestamp": "1970-01-01T00:00:00Z",
                "content": "hello",
                "scope": "global",
            }],
            "next_index": null,
        });
        let chunk = serde_json::from_value(json).unwrap();

        assert_eq!(
            ChatChunk {
                messages: vec![StoredMessage {
                    id: MessageId::nil(),
                    source: ParticipantId::nil(),
                    timestamp: Timestamp::unix_epoch(),
                    content: "hello".into(),
                    scope: Scope::Global,
                }],
                next_index: None,
            },
            chunk
        );
    }
}

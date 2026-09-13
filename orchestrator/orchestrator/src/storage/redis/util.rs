// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Context;
use redis::Value;

/// Helper type to make redis notification data easier to work with
///
/// This struct represents the data received from a Redis PMessage notification, which includes the
/// pattern, channel, and payload of the message.
pub(crate) struct PMessageData {
    /// The pattern that matched the channel for this message, e.g.
    /// "__keyspace@*__:ot-orchestrator:orchestrator:*:alive"
    pub(crate) pattern: String,
    /// The channel that the message was published to, e.g.
    /// "__keyspace@0__:ot-orchestrator:orchestrator:e13127e7-899a-4dcc-993a-28e5bc15cb11:alive"
    pub(crate) channel: String,
    /// The payload of the message, eg. "expired" or "set"
    pub(crate) payload: String,
}

impl TryFrom<Vec<Value>> for PMessageData {
    type Error = anyhow::Error;

    fn try_from(values: Vec<Value>) -> Result<Self, Self::Error> {
        if values.len() != 3 {
            anyhow::bail!(
                "Expected 3 elements in PMessage data, got {}: {:?}",
                values.len(),
                values
            );
        }

        let pattern = redis_to_string(&values[0]).context("Failed to parse PMessage 'pattern'")?;
        let channel = redis_to_string(&values[1]).context("Failed to parse PMessage 'channel'")?;
        let payload = redis_to_string(&values[2]).context("Failed to parse PMessage 'payload'")?;

        Ok(PMessageData {
            pattern,
            channel,
            payload,
        })
    }
}

impl std::fmt::Display for PMessageData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PMessageData {{ pattern: {}, channel: {}, payload: {} }}",
            self.pattern, self.channel, self.payload
        )
    }
}

pub fn redis_to_string(value: &redis::Value) -> anyhow::Result<String> {
    match value {
        redis::Value::BulkString(bytes) => String::from_utf8(bytes.clone())
            .context("Failed to build string from redis bulk string"),
        redis::Value::SimpleString(s) => Ok(s.clone()),
        _ => {
            anyhow::bail!("Unexpected Redis value type: {:?}", value);
        }
    }
}

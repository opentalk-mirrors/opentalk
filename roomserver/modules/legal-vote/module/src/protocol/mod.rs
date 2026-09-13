// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling protocol for the `legal-vote` namespace.

pub mod v1;

/// Represents a new protocol with version and protocol entries.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct NewProtocol {
    /// The version of the protocol.
    version: u8,

    /// A list of protocol entries.
    entries: Vec<v1::ProtocolEntry>,
}

impl NewProtocol {
    pub fn new(entries: Vec<v1::ProtocolEntry>) -> NewProtocol {
        Self {
            version: 1,
            entries,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use opentalk_roomserver_types_legal_vote::{token::Token, vote::VoteOption};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::protocol::v1::VoteEvent;

    #[test]
    fn serialization() {
        let produced = serde_json::to_value(NewProtocol {
            version: 1,
            entries: vec![v1::ProtocolEntry {
                timestamp: None,
                event: VoteEvent::Vote {
                    user_info: None,
                    token: Token::from_str("1111Cn8eVZg").unwrap(),
                    option: VoteOption::No,
                },
            }],
        })
        .unwrap();

        let expected = json!({
            "version": 1,
            "entries": [{
                "event": {
                    "event": "vote",
                    "token": "1111Cn8eVZg",
                    "option": "no",
                },
            }],
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization() {
        let produced: NewProtocol = serde_json::from_value(json!({
            "version": 1,
            "entries": [{
                "event": {
                    "event": "vote",
                    "token": "1111Cn8eVZg",
                    "option": "no",
                },
            }],
        }))
        .unwrap();

        let expected = NewProtocol {
            version: 1,
            entries: vec![v1::ProtocolEntry {
                timestamp: None,
                event: VoteEvent::Vote {
                    user_info: None,
                    token: Token::from_str("1111Cn8eVZg").unwrap(),
                    option: VoteOption::No,
                },
            }],
        };

        assert_eq!(produced, expected);
    }
}

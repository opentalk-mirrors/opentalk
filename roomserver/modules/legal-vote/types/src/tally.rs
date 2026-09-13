// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling tally for the `legal-vote` namespace.

use serde::{Deserialize, Serialize};

/// The vote options with their respective vote count.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tally {
    /// Vote count for yes.
    pub yes: u32,

    /// Vote count for no.
    pub no: u32,

    /// Vote count for abstain, abstain has to be enabled in the vote parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abstain: Option<u32>,
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialization() {
        let produced = serde_json::to_value(Tally {
            yes: 5,
            no: 8,
            abstain: Some(2),
        })
        .unwrap();

        let expected = json!({
            "yes": 5,
            "no": 8,
            "abstain": 2,
        });

        assert_eq!(produced, expected);

        let produced = serde_json::to_value(Tally {
            yes: 5,
            no: 8,
            abstain: None,
        })
        .unwrap();

        let expected = json!({
            "yes": 5,
            "no": 8,
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization() {
        let produced: Tally = serde_json::from_value(json!({
            "yes": 5,
            "no": 8,
            "abstain": 2,
        }))
        .unwrap();

        let expected = Tally {
            yes: 5,
            no: 8,
            abstain: Some(2),
        };

        assert_eq!(produced, expected);

        let produced: Tally = serde_json::from_value(json!({
            "yes": 5,
            "no": 8,
        }))
        .unwrap();

        let expected = Tally {
            yes: 5,
            no: 8,
            abstain: None,
        };

        assert_eq!(produced, expected);
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling issue messages for the `legal-vote` namespace.

mod technical_issue_kind;

use serde::{Deserialize, Serialize};
pub use technical_issue_kind::TechnicalIssueKind;

/// Represents an issue reported during the vote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Issue {
    /// A technical issue, such as audio or video problems.
    Technical {
        /// The kind of technical issue.
        kind: TechnicalIssueKind,

        /// A description of the technical issue, if available.
        ///
        /// Is `None` if no description is provided.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },

    /// A general issue that does not fall under technical problems.
    Other {
        /// A description of the reported issue.
        description: String,
    },
}

#[cfg(test)]
mod test {
    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_technical_issue() {
        let produced = serde_json::to_value(Issue::Technical {
            kind: TechnicalIssueKind::Audio,
            description: None,
        })
        .unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "kind": "audio"
        }
        "#);
    }

    #[test]
    fn deserialize_technical_issue() {
        let produced: Issue = serde_json::from_value(json!({
            "kind": "audio",
        }))
        .unwrap();

        let expected = Issue::Technical {
            kind: TechnicalIssueKind::Audio,
            description: None,
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_other_issue() {
        let produced = serde_json::to_value(Issue::Other {
            description: "Test Description".to_string(),
        })
        .unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "description": "Test Description"
        }
        "#);
    }

    #[test]
    fn deserialize_other_issue() {
        let produced: Issue = serde_json::from_value(json!({
            "description": "Test Description",
        }))
        .unwrap();

        let expected = Issue::Other {
            description: "Test Description".to_string(),
        };

        assert_eq!(produced, expected);
    }
}

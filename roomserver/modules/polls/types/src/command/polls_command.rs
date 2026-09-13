// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use serde::{Deserialize, Serialize};

use crate::{PollId, command::Vote, event::PollsEvent};

/// Commands received by the `polls` module
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum PollsCommand {
    /// Start a poll
    Start {
        /// The description of the poll topic
        topic: String,

        /// True if the poll is live
        #[serde(default)]
        live: bool,

        /// True if the poll accepts multiple choices
        #[serde(default)]
        multiple_choice: bool,

        /// The choices of the poll
        choices: Vec<String>,

        /// The duration of the poll
        #[serde(with = "opentalk_types_common::utils::duration_seconds")]
        duration: Duration,
    },

    /// Cast a vote
    Vote(Vote),

    /// Finish the poll
    Finish {
        /// The id of the poll
        id: PollId,
    },
}

impl CreateReplica<PollsEvent> for PollsCommand {
    fn replicate(&self) -> Option<PollsEvent> {
        match self {
            PollsCommand::Vote(vote) => Some(PollsEvent::Voted(vote.clone())),
            _ => None,
        }
    }
}

impl From<Vote> for PollsCommand {
    fn from(value: Vote) -> Self {
        Self::Vote(value)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, time::Duration};

    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::{ChoiceId, PollId, command::Choices};

    #[test]
    fn serialize_start() {
        let cmd = PollsCommand::Start {
            topic: "abc".to_string(),
            live: true,
            multiple_choice: false,
            choices: vec!["a".to_string(), "b".to_string()],
            duration: Duration::from_mins(5),
        };
        let raw = serde_json::to_string_pretty(&cmd).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "start",
          "topic": "abc",
          "live": true,
          "multiple_choice": false,
          "choices": [
            "a",
            "b"
          ],
          "duration": 300
        }
        "#);
    }

    #[test]
    fn deserialize_start() {
        let json = json!({
            "action": "start",
            "topic": "abc",
            "live": true,
            "choices": ["a", "b", "c"],
            "duration": 30
        });

        let message: PollsCommand = serde_json::from_value(json).unwrap();

        if let PollsCommand::Start {
            topic,
            live,
            multiple_choice,
            choices,
            duration,
        } = message
        {
            assert_eq!(topic, "abc");
            assert!(live);
            assert!(!multiple_choice);
            assert_eq!(choices, vec!["a", "b", "c"]);
            assert_eq!(duration, Duration::from_secs(30));
        } else {
            panic!("expected PollsCommand::Start but got: {message:?}");
        }
    }

    #[test]
    fn serialize_single_choice_vote() {
        let cmd = PollsCommand::Vote(Vote {
            poll_id: PollId::nil(),
            choices: Choices::Single {
                choice_id: ChoiceId::from_u32(1),
            },
        });
        let raw = serde_json::to_string_pretty(&cmd).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "vote",
          "poll_id": "00000000-0000-0000-0000-000000000000",
          "choice_id": 1
        }
        "#);
    }

    #[test]
    fn deserialize_single_choice_vote() {
        let json = json!({
           "action": "vote",
           "poll_id": "00000000-0000-0000-0000-000000000000",
           "choice_id": 321,
        });

        let message: PollsCommand = serde_json::from_value(json).unwrap();

        if let PollsCommand::Vote(Vote { poll_id, choices }) = message {
            assert_eq!(poll_id, PollId::nil());
            assert_eq!(
                choices,
                Choices::Single {
                    choice_id: ChoiceId::from(321),
                }
            );
        } else {
            panic!("Expected PollsCommand::Vote, got: {message:?}");
        }
    }

    #[test]
    fn serialize_multiple_choice_vote() {
        let cmd = PollsCommand::Vote(Vote {
            poll_id: PollId::nil(),
            choices: Choices::Multiple {
                choice_ids: BTreeSet::from_iter([ChoiceId::from_u32(0), ChoiceId::from_u32(1)]),
            },
        });
        let raw = serde_json::to_string_pretty(&cmd).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "vote",
          "poll_id": "00000000-0000-0000-0000-000000000000",
          "choice_ids": [
            0,
            1
          ]
        }
        "#);
    }

    #[test]
    fn deserialize_multiple_choice_vote() {
        let json = json!({
           "action": "vote",
           "poll_id": "00000000-0000-0000-0000-000000000000",
           "choice_ids": [322, 322, 323]
        });

        let message: PollsCommand = serde_json::from_value(json).unwrap();

        if let PollsCommand::Vote(Vote { poll_id, choices }) = message {
            assert_eq!(poll_id, PollId::nil());
            assert_eq!(
                choices,
                Choices::Multiple {
                    choice_ids: BTreeSet::from([ChoiceId::from(322), ChoiceId::from(323)]),
                }
            );
        } else {
            panic!("Expected PollsCommand::Vote, got: {message:?}");
        }
    }

    #[test]
    fn serialize_conflicting_choice_vote() {
        let json = json!({
           "action": "vote",
           "poll_id": "00000000-0000-0000-0000-000000000000",
           "choice_id": 321,
           "choice_ids": [322, 322, 323]
        });

        let message: PollsCommand = serde_json::from_value(json).unwrap();

        if let PollsCommand::Vote(Vote { poll_id, choices }) = message {
            assert_eq!(poll_id, PollId::nil());
            assert_eq!(
                choices,
                Choices::Single {
                    choice_id: ChoiceId::from(321),
                }
            );
        } else {
            panic!("Expected PollsCommand::Vote, got: {message:?}");
        }
    }

    #[test]
    fn deserialize_abstain() {
        let json = json!({
           "action": "vote",
           "poll_id": "00000000-0000-0000-0000-000000000000"
        });

        let message: PollsCommand = serde_json::from_value(json).unwrap();

        if let PollsCommand::Vote(Vote { poll_id, choices }) = message {
            assert_eq!(poll_id, PollId::nil());
            assert_eq!(
                choices,
                Choices::Multiple {
                    choice_ids: BTreeSet::new(),
                }
            );
        } else {
            panic!("Expected PollsCommand::Vote, got: {message:?}");
        }
    }

    #[test]
    fn serialize_finish() {
        let cmd = PollsCommand::Finish { id: PollId::nil() };
        let raw = serde_json::to_string_pretty(&cmd).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "finish",
          "id": "00000000-0000-0000-0000-000000000000"
        }
        "#);
    }

    #[test]
    fn deserialize_finish() {
        let json = json!({
            "action": "finish",
            "id": "00000000-0000-0000-0000-000000000000",
        });

        let message: PollsCommand = serde_json::from_value(json).unwrap();

        assert_eq!(message, PollsCommand::Finish { id: PollId::nil() });
    }
}

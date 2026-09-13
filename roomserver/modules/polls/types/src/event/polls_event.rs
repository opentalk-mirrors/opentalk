// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{Choice, PollId, Results, command::Vote, event::Error};

/// Events sent out by the `polls` module
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "message")]
pub enum PollsEvent {
    /// The poll has started
    Started {
        /// The id of the poll
        id: PollId,

        /// The description of the poll topic
        topic: String,

        /// True if the poll is live
        live: bool,

        /// True if the poll accepts multiple choices
        multiple_choice: bool,

        /// Choices of the poll
        choices: Vec<Choice>,

        /// Duration of the poll
        #[serde(with = "opentalk_types_common::utils::duration_seconds")]
        duration: Duration,
    },

    /// Live update of the poll results
    LiveUpdate(Results),

    /// A vote was cast on a different device
    Voted(Vote),

    /// The poll is completed
    Done(Results),

    /// An error happened when executing a `polls` command
    Error(Error),
}

impl From<Error> for PollsEvent {
    fn from(value: Error) -> Self {
        Self::Error(value)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, time::Duration};

    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::{Choice, ChoiceId, Item, PollId, command::Choices};

    #[test]
    fn serialize_started() {
        let started = PollsEvent::Started {
            id: PollId::nil(),
            topic: "polling".into(),
            live: true,
            multiple_choice: false,
            choices: vec![
                Choice {
                    id: ChoiceId::from(0),
                    content: "yes".into(),
                },
                Choice {
                    id: ChoiceId::from(1),
                    content: "no".into(),
                },
            ],
            duration: Duration::from_secs(10),
        };

        assert_snapshot!(
            serde_json::to_string_pretty(&started).unwrap(),
            @r#"
        {
          "message": "started",
          "id": "00000000-0000-0000-0000-000000000000",
          "topic": "polling",
          "live": true,
          "multiple_choice": false,
          "choices": [
            {
              "id": 0,
              "content": "yes"
            },
            {
              "id": 1,
              "content": "no"
            }
          ],
          "duration": 10
        }
        "#
        );
    }

    #[test]
    fn deserialize_started() {
        let produced: PollsEvent = serde_json::from_value(json!({
            "message": "started",
            "id": "00000000-0000-0000-0000-000000000000",
            "topic": "polling",
            "live": true,
            "multiple_choice": false,
            "choices": [
                {
                    "id": 0,
                    "content": "yes"
                },
                {
                    "id": 1,
                    "content": "no"
                }
            ],
            "duration": 10
        }))
        .unwrap();

        let expected = PollsEvent::Started {
            id: PollId::nil(),
            topic: "polling".to_string(),
            live: true,
            multiple_choice: false,
            choices: vec![
                Choice {
                    id: ChoiceId::from_u32(0),
                    content: "yes".to_string(),
                },
                Choice {
                    id: ChoiceId::from_u32(1),
                    content: "no".to_string(),
                },
            ],
            duration: Duration::from_secs(10),
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_live_update() {
        let live_update = PollsEvent::LiveUpdate(Results {
            id: PollId::nil(),
            results: vec![
                Item {
                    id: ChoiceId::from(0),
                    count: 32,
                },
                Item {
                    id: ChoiceId::from(1),
                    count: 64,
                },
            ],
        });

        assert_snapshot!(
            serde_json::to_string_pretty(&live_update).unwrap(),
            @r#"
        {
          "message": "live_update",
          "id": "00000000-0000-0000-0000-000000000000",
          "results": [
            {
              "id": 0,
              "count": 32
            },
            {
              "id": 1,
              "count": 64
            }
          ]
        }
        "#
        );
    }

    #[test]
    fn deserialize_live_update() {
        let produced: PollsEvent = serde_json::from_value(json!({
            "message": "live_update",
            "id": "00000000-0000-0000-0000-000000000000",
            "results": [
                {
                    "id": 0,
                    "count": 32
                },
                {
                  "id": 1,
                  "count": 64
                }
            ]
        }))
        .unwrap();

        let expected = PollsEvent::LiveUpdate(Results {
            id: PollId::nil(),
            results: vec![
                Item {
                    id: ChoiceId::from_u32(0),
                    count: 32,
                },
                Item {
                    id: ChoiceId::from_u32(1),
                    count: 64,
                },
            ],
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_voted() {
        let voted = PollsEvent::Voted(Vote {
            poll_id: PollId::nil(),
            choices: Choices::Multiple {
                choice_ids: BTreeSet::from([ChoiceId::from_u32(0), ChoiceId::from_u32(1)]),
            },
        });

        assert_snapshot!(
            serde_json::to_string_pretty(&voted).unwrap(),
            @r#"
        {
          "message": "voted",
          "poll_id": "00000000-0000-0000-0000-000000000000",
          "choice_ids": [
            0,
            1
          ]
        }
        "#
        )
    }

    #[test]
    fn deserialize_voted() {
        let produced: PollsEvent = serde_json::from_value(json!({
           "message": "voted",
            "poll_id": "00000000-0000-0000-0000-000000000000",
            "choice_ids": [
                0,
                1
            ]
        }))
        .unwrap();

        let expected = PollsEvent::Voted(Vote {
            poll_id: PollId::nil(),
            choices: Choices::Multiple {
                choice_ids: BTreeSet::from_iter([ChoiceId::from(0), ChoiceId::from(1)]),
            },
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_done() {
        let done = PollsEvent::Done(Results {
            id: PollId::nil(),
            results: vec![
                Item {
                    id: ChoiceId::from(0),
                    count: 32,
                },
                Item {
                    id: ChoiceId::from(1),
                    count: 64,
                },
            ],
        });

        assert_snapshot!(
            serde_json::to_string_pretty(&done).unwrap(),
            @r#"
        {
          "message": "done",
          "id": "00000000-0000-0000-0000-000000000000",
          "results": [
            {
              "id": 0,
              "count": 32
            },
            {
              "id": 1,
              "count": 64
            }
          ]
        }
        "#
        );
    }

    #[test]
    fn deserialize_done() {
        let produced: PollsEvent = serde_json::from_value(json!({
            "message": "done",
            "id": "00000000-0000-0000-0000-000000000000",
            "results": [
              {
                "id": 0,
                "count": 32
              },
              {
                "id": 1,
                "count": 64
              }
            ]
        }))
        .unwrap();

        let expected = PollsEvent::Done(Results {
            id: PollId::nil(),
            results: vec![
                Item {
                    id: ChoiceId::from_u32(0),
                    count: 32,
                },
                Item {
                    id: ChoiceId::from_u32(1),
                    count: 64,
                },
            ],
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_insufficient_permissions() {
        let error = PollsEvent::Error(Error::InsufficientPermissions);
        let produced = serde_json::to_string_pretty(&error).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "error",
          "error": "insufficient_permissions"
        }
        "#);
    }

    #[test]
    fn deserialize_insufficient_permissions() {
        let produced: PollsEvent = serde_json::from_value(json!({
            "message": "error",
            "error": "insufficient_permissions"
        }))
        .unwrap();
        let expected = PollsEvent::Error(Error::InsufficientPermissions);

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_invalid_choice_count() {
        let error = PollsEvent::Error(Error::InvalidChoiceCount {
            min_choice_count: 1,
            max_choice_count: 2,
        });
        let produced = serde_json::to_string_pretty(&error).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "error",
          "error": "invalid_choice_count",
          "min_choice_count": 1,
          "max_choice_count": 2
        }
        "#);
    }

    #[test]
    fn deserialize_invalid_choice_count() {
        let produced: PollsEvent = serde_json::from_value(json!({
            "message": "error",
            "error": "invalid_choice_count",
            "min_choice_count": 1,
            "max_choice_count": 2
        }))
        .unwrap();
        let expected = PollsEvent::Error(Error::InvalidChoiceCount {
            min_choice_count: 1,
            max_choice_count: 2,
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_invalid_poll_id() {
        let error = PollsEvent::Error(Error::InvalidPollId);
        let produced = serde_json::to_string_pretty(&error).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "error",
          "error": "invalid_poll_id"
        }
        "#);
    }

    #[test]
    fn deserialize_invalid_poll_id() {
        let produced: PollsEvent = serde_json::from_value(json!({
            "message": "error",
            "error": "invalid_poll_id"
        }))
        .unwrap();
        let expected = PollsEvent::Error(Error::InvalidPollId);

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_invalid_choice_id() {
        let error = PollsEvent::Error(Error::InvalidChoiceId);
        let produced = serde_json::to_string_pretty(&error).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "error",
          "error": "invalid_choice_id"
        }
        "#);
    }

    #[test]
    fn deserialize_invalid_choice_id() {
        let produced: PollsEvent = serde_json::from_value(json!({
            "message": "error",
            "error": "invalid_choice_id"
        }))
        .unwrap();
        let expected = PollsEvent::Error(Error::InvalidChoiceId);

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_multiple_choices_not_allowed() {
        let error = PollsEvent::Error(Error::MultipleChoicesNotAllowed);
        let produced = serde_json::to_string_pretty(&error).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "error",
          "error": "multiple_choices_not_allowed"
        }
        "#);
    }

    #[test]
    fn deserialize_multiple_choices_not_allowed() {
        let produced: PollsEvent = serde_json::from_value(json!({
            "message": "error",
            "error": "multiple_choices_not_allowed"
        }))
        .unwrap();
        let expected = PollsEvent::Error(Error::MultipleChoicesNotAllowed);

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_invalid_choice_description_length() {
        let error = PollsEvent::Error(Error::InvalidChoiceDescriptionLength {
            min_length: 1,
            max_length: 2,
        });
        let produced = serde_json::to_string_pretty(&error).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "error",
          "error": "invalid_choice_description_length",
          "min_length": 1,
          "max_length": 2
        }
        "#);
    }

    #[test]
    fn deserialize_invalid_choice_description_length() {
        let produced: PollsEvent = serde_json::from_value(json!({
            "message": "error",
            "error": "invalid_choice_description_length",
            "min_length": 1,
            "max_length": 2
        }))
        .unwrap();
        let expected = PollsEvent::Error(Error::InvalidChoiceDescriptionLength {
            min_length: 1,
            max_length: 2,
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_invalid_duration() {
        let error = PollsEvent::Error(Error::InvalidDuration {
            max_duration: Duration::from_mins(5),
        });
        let produced = serde_json::to_string_pretty(&error).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "error",
          "error": "invalid_duration",
          "max_duration": 300
        }
        "#);
    }

    #[test]
    fn deserialize_invalid_duration() {
        let produced: PollsEvent = serde_json::from_value(json!({
            "message": "error",
            "error": "invalid_duration",
            "max_duration": 300
        }))
        .unwrap();
        let expected = PollsEvent::Error(Error::InvalidDuration {
            max_duration: Duration::from_mins(5),
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_invalid_topic_length() {
        let error = PollsEvent::Error(Error::InvalidTopicLength { max_length: 100 });
        let produced = serde_json::to_string_pretty(&error).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "error",
          "error": "invalid_topic_length",
          "max_length": 100
        }
        "#);
    }

    #[test]
    fn deserialize_invalid_topic_length() {
        let produced: PollsEvent = serde_json::from_value(json!({
            "message": "error",
            "error": "invalid_topic_length",
            "max_length": 100
        }))
        .unwrap();
        let expected = PollsEvent::Error(Error::InvalidTopicLength { max_length: 100 });

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_still_running() {
        let error = PollsEvent::Error(Error::InsufficientPermissions);
        let produced = serde_json::to_string_pretty(&error).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "error",
          "error": "insufficient_permissions"
        }
        "#);
    }

    #[test]
    fn deserialize_still_running() {
        let produced: PollsEvent = serde_json::from_value(json!({
            "message": "error",
            "error": "still_running"
        }))
        .unwrap();
        let expected = PollsEvent::Error(Error::StillRunning);

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_internal() {
        let error = PollsEvent::Error(Error::Internal);
        let produced = serde_json::to_string_pretty(&error).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "error",
          "error": "internal"
        }
        "#);
    }

    #[test]
    fn deserialize_internal() {
        let produced: PollsEvent = serde_json::from_value(json!({
            "message": "error",
            "error": "internal"
        }))
        .unwrap();
        let expected = PollsEvent::Error(Error::Internal);

        assert_eq!(produced, expected);
    }
}

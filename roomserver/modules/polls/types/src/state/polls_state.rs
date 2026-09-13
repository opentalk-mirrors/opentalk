// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Frontend data for `polls` namespace

use std::time::Duration;

use chrono::Utc;
use opentalk_types_common::time::Timestamp;
use serde::{Deserialize, Serialize};

use crate::{Choice, PollId};

/// The state of the `polls` module.
///
/// This struct is sent to the participant in the `join_success` message
/// when they join successfully to the meeting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PollsState {
    /// The id of the poll
    pub id: PollId,

    /// The description of the poll topic
    pub topic: String,

    /// True if the poll is live
    pub live: bool,

    /// True if the poll accepts multiple choices
    pub multiple_choice: bool,

    /// Choices of the poll
    pub choices: Vec<Choice>,

    /// The time when the poll started
    pub started: Timestamp,

    /// The duration of the poll
    #[serde(with = "opentalk_types_common::utils::duration_seconds")]
    pub duration: Duration,
}

impl opentalk_types_signaling::SignalingModuleFrontendData for PollsState {
    const NAMESPACE: Option<opentalk_types_common::modules::ModuleId> =
        Some(crate::POLLS_MODULE_ID);
}

impl PollsState {
    /// Get the remaining duration of the poll
    pub fn remaining(&self) -> Option<Duration> {
        let duration = chrono::Duration::from_std(self.duration)
            .expect("duration as secs should never be larger than i64::MAX");

        let expire = (*self.started) + duration;
        let now = Utc::now();

        // difference will be negative duration if expired.
        // Conversion to std duration will fail -> returning None
        (expire - now).to_std().ok()
    }

    /// Is the poll expired
    pub fn is_expired(&self) -> bool {
        self.remaining().is_none()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use insta::assert_snapshot;
    use opentalk_types_common::time::Timestamp;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::PollsState;
    use crate::{Choice, ChoiceId, PollId};

    #[test]
    fn serialize_polls_state() {
        let polls_state = PollsState {
            id: PollId::nil(),
            topic: "A or B?".to_string(),
            live: true,
            multiple_choice: false,
            choices: vec![
                Choice {
                    id: ChoiceId::from_u32(0),
                    content: "A".to_string(),
                },
                Choice {
                    id: ChoiceId::from_u32(1),
                    content: "B".to_string(),
                },
            ],
            started: Timestamp::unix_epoch(),
            duration: Duration::from_mins(5),
        };

        let produced = serde_json::to_string_pretty(&polls_state).unwrap();
        assert_snapshot!(produced, @r#"
        {
          "id": "00000000-0000-0000-0000-000000000000",
          "topic": "A or B?",
          "live": true,
          "multiple_choice": false,
          "choices": [
            {
              "id": 0,
              "content": "A"
            },
            {
              "id": 1,
              "content": "B"
            }
          ],
          "started": "1970-01-01T00:00:00Z",
          "duration": 300
        }
        "#);
    }

    #[test]
    fn deserialize_polls_state() {
        let produced: PollsState = serde_json::from_value(json!({
            "id": "00000000-0000-0000-0000-000000000000",
            "topic": "A or B?",
            "live": true,
            "multiple_choice": false,
            "choices": [
                {"id": 0, "content": "A"},
                {"id": 1, "content": "B"}
            ],
            "started": "1970-01-01T00:00:00Z",
            "duration": 300
        }))
        .unwrap();

        let expected = PollsState {
            id: PollId::nil(),
            topic: "A or B?".to_string(),
            live: true,
            multiple_choice: false,
            choices: vec![
                Choice {
                    id: ChoiceId::from_u32(0),
                    content: "A".to_string(),
                },
                Choice {
                    id: ChoiceId::from_u32(1),
                    content: "B".to_string(),
                },
            ],
            started: Timestamp::unix_epoch(),
            duration: Duration::from_mins(5),
        };

        assert_eq!(produced, expected);
    }
}

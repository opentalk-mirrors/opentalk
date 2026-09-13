// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::{
    config::PublicConfig,
    event::{AutomodError, StoppedReason},
};

/// Events send by the `automod` module
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "message")]
pub enum AutomodEvent {
    /// Signals the start of an automod session
    Started(PublicConfig),

    /// Signals the end of an automod session
    ///
    /// See [`StoppedReason`]
    Stopped(StoppedReason),

    /// The current speaker state has changed
    ///
    /// This event will ALWAYS notify of a speaker change, even if the speaker is the same
    /// participant as before, it MUST be handled as changed.
    ///
    /// Both `history` and `remaining`: If the field is set it will contain the complete new list.
    /// If it doesn't exist it must be treated as unchanged.
    SpeakerUpdated {
        /// Speaker field. If [`None`] no speaker is currently selected.
        #[serde(skip_serializing_if = "Option::is_none")]
        speaker: Option<ParticipantId>,

        /// Optional modification of the history.
        ///
        /// If set the frontend MUST replace its history with the given one.
        /// If not set the frontend MUST keep its current history.
        #[serde(skip_serializing_if = "Option::is_none")]
        history: Option<Vec<ParticipantId>>,

        /// Optional modification of the remaining participants.
        ///
        /// Remaining participants must be interpreted differently depending on the selection
        /// strategy. E.g. in the playlist moderation `remaining` lists the participants
        /// left inside the playlist. All other strategies will use `remaining` (if at all)
        /// to list all participants (if public) that are eligible to be selected.
        ///
        /// This will only be set when using the `playlist` selection_strategy.
        ///
        /// If set the frontend MUST replace its remaining list with the given one.
        /// If not set the frontend MUST keep its current remaining list.
        #[serde(skip_serializing_if = "Option::is_none")]
        remaining: Option<Vec<ParticipantId>>,
    },

    /// The remaining list has been updated
    RemainingUpdated {
        /// List of remaining participants in selection pool
        remaining: Vec<ParticipantId>,
    },

    /// An error has occurred
    ///
    /// See [`AutomodError`]
    Error(AutomodError),
}

impl From<AutomodError> for AutomodEvent {
    fn from(err: AutomodError) -> Self {
        Self::Error(err)
    }
}

#[cfg(test)]
mod tests {
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::config::{FrontendConfig, Parameter, SelectionStrategy};

    #[test]
    fn started_event() {
        let produced = serde_json::to_value(AutomodEvent::Started(
            FrontendConfig {
                parameter: Parameter {
                    selection_strategy: SelectionStrategy::None,
                    show_remaining: false,
                    time_limit: None,
                    allow_double_selection: false,
                    auto_append_on_join: false,
                },
                history: vec![ParticipantId::from_u128(1)],
                remaining: vec![ParticipantId::from_u128(2)],
                issued_by: ParticipantId::from_u128(3),
            }
            .into_public(),
        ))
        .unwrap();

        let expected = json!({
            "message": "started",
            "selection_strategy": "none",
            "show_remaining": false,
            "allow_double_selection": false,
            "auto_append_on_join": false,
            "history": ["00000000-0000-0000-0000-000000000001"],
            "remaining": ["00000000-0000-0000-0000-000000000002"],
            "issued_by": "00000000-0000-0000-0000-000000000003",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn stopped_by_moderator_event() {
        let produced =
            serde_json::to_value(AutomodEvent::Stopped(StoppedReason::StoppedByModerator {
                issued_by: ParticipantId::from_u128(1),
            }))
            .unwrap();

        let expected = json!({
            "message": "stopped",
            "reason": "stopped_by_moderator",
            "issued_by": "00000000-0000-0000-0000-000000000001"
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn stopped_session_finished_event() {
        let produced =
            serde_json::to_value(AutomodEvent::Stopped(StoppedReason::SessionFinished)).unwrap();

        let expected = json!({
            "message": "stopped",
            "reason": "session_finished",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn speaker_update_event() {
        let produced = serde_json::to_value(AutomodEvent::SpeakerUpdated {
            speaker: Some(ParticipantId::from_u128(1)),
            history: Some(vec![]),
            remaining: Some(vec![ParticipantId::from_u128(2)]),
        })
        .unwrap();

        let expected = json!({
            "message": "speaker_updated",
            "speaker": "00000000-0000-0000-0000-000000000001",
            "history": [],
            "remaining": ["00000000-0000-0000-0000-000000000002"]
        });

        assert_eq!(produced, expected);

        let produced = serde_json::to_value(AutomodEvent::SpeakerUpdated {
            speaker: None,
            history: None,
            remaining: None,
        })
        .unwrap();

        let expected = json!({
            "message": "speaker_updated",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn remaining_update_event() {
        let produced = serde_json::to_value(AutomodEvent::RemainingUpdated {
            remaining: vec![ParticipantId::from_u128(1), ParticipantId::from_u128(2)],
        })
        .unwrap();

        let expected = json!({
            "message": "remaining_updated",
            "remaining": [
                "00000000-0000-0000-0000-000000000001",
                "00000000-0000-0000-0000-000000000002"
            ]
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn error_invalid_selection_event() {
        let produced =
            serde_json::to_value(AutomodEvent::Error(AutomodError::InvalidSelection)).unwrap();

        let expected = json!({
            "message": "error",
            "error": "invalid_selection"
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn error_invalid_insufficient_permissions_event() {
        let produced =
            serde_json::to_value(AutomodEvent::Error(AutomodError::InsufficientPermissions))
                .unwrap();

        let expected = json!({
            "message": "error",
            "error": "insufficient_permissions"
        });

        assert_eq!(produced, expected);
    }
}

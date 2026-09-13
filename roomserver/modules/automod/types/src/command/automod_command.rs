use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2
use crate::{command::Select, config::Parameter, event::AutomodEvent};

/// Commands received by the `automod` module
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum AutomodCommand {
    /// Start the auto-moderation with a provided config
    Start {
        /// The parameters for the automod session
        #[serde(flatten)]
        parameter: Parameter,

        /// Depending on the selection strategy, the list of Participant that can be chosen from.
        ///
        ///
        /// - Strategy = `none`, `random` or `nomination`: The allow_list acts as pool of
        ///   participants which can be selected (by nomination or randomly etc).
        ///
        /// - Strategy = `playlist` The allow_list does not get used by this strategy.
        #[serde(skip_serializing_if = "Option::is_none", default)]
        allow_list: Option<Vec<ParticipantId>>,

        /// Ordered list of queued participants
        ///
        /// - Strategy = `none`, `random` or `nomination`: The playlist does not get used by these
        ///   strategies.
        ///
        /// - Strategy = `playlist` The playlist is a ordered list of participants which will get
        ///   used to select the next participant when yielding. It is also used as a pool to
        ///   select participants randomly from (moderator command `Select`).
        #[serde(skip_serializing_if = "Option::is_none", default)]
        playlist: Option<Vec<ParticipantId>>,
    },

    /// Set either the allow_list or playlist or both
    Edit {
        /// Edit the `allow_list`. If `None`, it should not be edited.
        #[serde(skip_serializing_if = "Option::is_none", default)]
        allow_list: Option<Vec<ParticipantId>>,

        /// Edit the `playlist`. If `None`, it should not be edited.
        #[serde(skip_serializing_if = "Option::is_none", default)]
        playlist: Option<Vec<ParticipantId>>,
    },

    /// Stop the auto-moderation
    Stop,

    /// Select a user to be active speaker
    Select(Select),

    /// User yields it's speaker status
    Yield {
        /// In some cases a user must select the next participant to be speaker
        #[serde(skip_serializing_if = "Option::is_none", default)]
        next: Option<ParticipantId>,
    },
}

impl AutomodCommand {
    /// Returns if the issued command requires the participant to be a moderator
    pub fn requires_moderator_privileges(&self) -> bool {
        match self {
            AutomodCommand::Start { .. }
            | AutomodCommand::Edit { .. }
            | AutomodCommand::Stop
            | AutomodCommand::Select(_) => true,
            AutomodCommand::Yield { .. } => false,
        }
    }
}

impl CreateReplica<AutomodEvent> for AutomodCommand {
    fn replicate(&self) -> Option<AutomodEvent> {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::config::{Parameter, SelectionStrategy};

    #[test]
    fn requires_moderator_privileges() {
        let start = AutomodCommand::Start {
            parameter: Parameter {
                selection_strategy: SelectionStrategy::None,
                show_remaining: false,
                time_limit: None,
                allow_double_selection: false,
                auto_append_on_join: false,
            },
            allow_list: None,
            playlist: None,
        };

        let edit = AutomodCommand::Edit {
            allow_list: None,
            playlist: None,
        };

        let stop = AutomodCommand::Stop;

        let select = AutomodCommand::Select(Select::None);

        let r#yield = AutomodCommand::Yield { next: None };

        assert!(start.requires_moderator_privileges());
        assert!(edit.requires_moderator_privileges());
        assert!(stop.requires_moderator_privileges());
        assert!(select.requires_moderator_privileges());

        assert!(!r#yield.requires_moderator_privileges());
    }

    #[test]
    fn start_command() {
        let produced = serde_json::to_value(AutomodCommand::Start {
            parameter: Parameter {
                selection_strategy: SelectionStrategy::None,
                show_remaining: false,
                time_limit: Some(Duration::from_millis(5000)),
                allow_double_selection: false,
                auto_append_on_join: false,
            },
            allow_list: Some(vec![ParticipantId::from_u128(1)]),
            playlist: Some(vec![ParticipantId::from_u128(2)]),
        })
        .unwrap();

        let expected = json!({
            "action": "start",
            "selection_strategy": "none",
            "show_remaining": false,
            "time_limit": 5000,
            "allow_double_selection": false,
            "auto_append_on_join": false,
            "allow_list": ["00000000-0000-0000-0000-000000000001"],
            "playlist": ["00000000-0000-0000-0000-000000000002"]
        });

        assert_eq!(produced, expected);

        let produced = serde_json::to_value(AutomodCommand::Start {
            parameter: Parameter {
                selection_strategy: SelectionStrategy::None,
                show_remaining: false,
                time_limit: None,
                allow_double_selection: false,
                auto_append_on_join: false,
            },
            allow_list: None,
            playlist: None,
        })
        .unwrap();

        let expected = json!({
            "action": "start",
            "selection_strategy": "none",
            "show_remaining": false,
            "allow_double_selection": false,
            "auto_append_on_join": false,
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn edit_command() {
        let produced = serde_json::to_value(AutomodCommand::Edit {
            allow_list: Some(vec![ParticipantId::from_u128(1)]),
            playlist: Some(vec![ParticipantId::from_u128(2)]),
        })
        .unwrap();

        let expected = json!({
            "action": "edit",
            "allow_list": ["00000000-0000-0000-0000-000000000001"],
            "playlist": ["00000000-0000-0000-0000-000000000002"],
        });

        assert_eq!(produced, expected);

        let produced = serde_json::to_value(AutomodCommand::Edit {
            allow_list: None,
            playlist: None,
        })
        .unwrap();

        let expected = json!({
            "action": "edit",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn stop_command() {
        let produced = serde_json::to_value(AutomodCommand::Stop).unwrap();

        let expected = json!({
            "action": "stop"
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn select_none_command() {
        let produced = serde_json::to_value(AutomodCommand::Select(Select::None)).unwrap();

        let expected = json!({
            "action": "select",
            "how": "none"
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn select_random_command() {
        let produced = serde_json::to_value(AutomodCommand::Select(Select::Random)).unwrap();

        let expected = json!({
            "action": "select",
            "how": "random"
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn select_next_command() {
        let produced = serde_json::to_value(AutomodCommand::Select(Select::Next)).unwrap();

        let expected = json!({
            "action": "select",
            "how": "next",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn select_specific_command() {
        let produced = serde_json::to_value(AutomodCommand::Select(Select::Specific {
            participant: ParticipantId::from_u128(1),
            keep_in_remaining: false,
        }))
        .unwrap();

        let expected = json!({
            "action": "select",
            "how": "specific",
            "participant": "00000000-0000-0000-0000-000000000001",
            "keep_in_remaining": false,
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn yield_command() {
        let produced = serde_json::to_value(AutomodCommand::Yield {
            next: Some(ParticipantId::from_u128(1)),
        })
        .unwrap();

        let expected = json!({
            "action": "yield",
            "next": "00000000-0000-0000-0000-000000000001",
        });

        assert_eq!(produced, expected);

        let produced = serde_json::to_value(AutomodCommand::Yield { next: None }).unwrap();

        let expected = json!({
            "action": "yield",
        });

        assert_eq!(produced, expected);
    }
}

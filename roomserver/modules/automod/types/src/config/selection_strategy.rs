// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

/// The `SelectionStrategy` configured by the moderator, which determines, how the next participant
/// is chosen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SelectionStrategy {
    /// No selection strategy, a moderator will assign privileges
    None,

    /// The next participant is the one next in the list
    Playlist,

    /// The next participant is randomly chosen
    Random,

    /// The current participant will nominate the next one
    Nomination,
}

impl SelectionStrategy {
    /// Determines if `playlist` is used to describe pool of valid selection targets
    pub fn uses_playlist(self) -> bool {
        matches!(self, SelectionStrategy::Playlist)
    }

    /// Determines if the `allow_list` is used to describe pool of valid selection targets
    pub fn uses_allow_list(self) -> bool {
        !self.uses_playlist()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn uses_playlist() {
        let none = SelectionStrategy::None;

        let playlist = SelectionStrategy::Playlist;

        let random = SelectionStrategy::Random;

        let nomination = SelectionStrategy::Nomination;

        assert!(playlist.uses_playlist());

        assert!(!none.uses_playlist());
        assert!(!random.uses_playlist());
        assert!(!nomination.uses_playlist());
    }

    #[test]
    fn uses_allow_list() {
        let none = SelectionStrategy::None;

        let playlist = SelectionStrategy::Playlist;

        let random = SelectionStrategy::Random;

        let nomination = SelectionStrategy::Nomination;

        assert!(none.uses_allow_list());
        assert!(random.uses_allow_list());
        assert!(nomination.uses_allow_list());

        assert!(!playlist.uses_allow_list());
    }

    #[test]
    fn selection_strategy_none() {
        let produced = serde_json::to_value(SelectionStrategy::None).unwrap();
        let expected = json!("none");

        assert_eq!(produced, expected);
    }

    #[test]
    fn selection_strategy_playlist() {
        let produced = serde_json::to_value(SelectionStrategy::Playlist).unwrap();
        let expected = json!("playlist");

        assert_eq!(produced, expected);
    }

    #[test]
    fn selection_strategy_random() {
        let produced = serde_json::to_value(SelectionStrategy::Random).unwrap();
        let expected = json!("random");

        assert_eq!(produced, expected);
    }

    #[test]
    fn selection_strategy_nomination() {
        let produced = serde_json::to_value(SelectionStrategy::Nomination).unwrap();
        let expected = json!("nomination");

        assert_eq!(produced, expected);
    }
}

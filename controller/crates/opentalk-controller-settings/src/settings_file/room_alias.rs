// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RoomAlias {
    /// When `true`, rooms with an alias will not have a suffix appended to their name. This makes the rooms vulnerable
    /// to brute-force attacks and is therefore considered **insecure** when untrusted people can reach your OpenTalk
    /// instance and should only be enabled in trusted environments.
    pub disable_suffix: bool,

    // TODO: might have to update max suffix length, https://git.opentalk.dev/opentalk/product/tickets/-/work_items/268#note_611830
    /// The number of characters the suffix appended to a room name has. The suffix is a randomly generated string that
    /// protects the room alias against brute-force guessing. This is only relevant when `disable_suffix` is `false`
    /// (default).
    /// Short suffixes are vulnerable to brute-force attacks, so it is recommended to use a length of at least 16
    /// characters.
    ///
    /// Allowed values are between 8 and 64, inclusive.
    #[serde(default = "default_suffix_length")]
    pub suffix_length: u8,
}

impl Default for RoomAlias {
    fn default() -> Self {
        Self {
            disable_suffix: false,
            suffix_length: default_suffix_length(),
        }
    }
}

const fn default_suffix_length() -> u8 {
    16
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{Display, From, FromStr, Into};

use crate::utils::ExampleData;

/// A ticket token
#[derive(Display, From, FromStr, Into, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct TicketToken(String);

impl TicketToken {
    /// Generate a ticket for a room, based on random data.
    #[cfg(feature = "rand")]
    pub fn generate_for_room(room: crate::rooms::RoomId) -> Self {
        // Make 64 byte long string
        // {uuid}#{random_chars}

        use rand::RngExt as _;
        Self(
            room.to_string()
                .chars()
                .chain(Some('#'))
                .chain(
                    rand::rng()
                        .sample_iter(rand::distr::Alphanumeric)
                        .take(27) // uuid has a length of 36, add 27 random chars
                        .map(char::from),
                )
                .collect(),
        )
    }

    /// Get a str reference to the data in the ticket token
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ExampleData for TicketToken {
    fn example_data() -> Self {
        Self("abcdefghijklmnopqrstuvwxyz123456abcdefghijklmnopqrstuvwxyz123456".to_string())
    }
}

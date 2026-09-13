// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Display, From, FromStr, Into};
use serde::{Deserialize, Serialize};

/// The id of the Choice
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    FromStr,
    AsRef,
    Display,
    From,
    Into,
    Serialize,
    Deserialize,
)]
pub struct ChoiceId(u32);

impl ChoiceId {
    /// Create a new ChoiceId
    pub const fn from_u32(id: u32) -> Self {
        Self(id)
    }
}

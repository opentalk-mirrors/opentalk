// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{Display, From, FromStr, Into};

/// A bearer token
#[derive(Display, From, FromStr, Into, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BearerToken(String);

impl BearerToken {
    /// Create a new bearer token
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }
}

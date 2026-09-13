// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{Display, From, FromStr, Into};

/// A logout token
#[derive(Display, From, FromStr, Into, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct LogoutToken(String);

impl LogoutToken {
    /// Get a str reference to the data in the ticket token
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

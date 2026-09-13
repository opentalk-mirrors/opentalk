// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{Display, From, FromStr, Into};

use crate::utils::ExampleData;

/// A resumption token
#[derive(Display, From, FromStr, Into, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ResumptionToken(String);

impl ResumptionToken {
    /// Generate a new random resumption token
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        use rand::RngExt;

        let token = rand::rng()
            .sample_iter(rand::distr::Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        Self(token)
    }
}

impl ExampleData for ResumptionToken {
    fn example_data() -> Self {
        Self("654321zyxwvutsrqponmlkjihgfedcba654321zyxwvutsrqponmlkjihgfedcba".to_string())
    }
}

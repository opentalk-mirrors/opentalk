// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use anyhow::bail;
use rand::{RngExt, distr::Alphanumeric};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(try_from = "String")]
pub struct SignalingSalt(pub String);

impl SignalingSalt {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn generate() -> SignalingSalt {
        let salt = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(24)
            .map(char::from)
            .collect();

        SignalingSalt(salt)
    }
}

impl Default for SignalingSalt {
    fn default() -> Self {
        Self::generate()
    }
}

impl TryFrom<String> for SignalingSalt {
    type Error = anyhow::Error;

    fn try_from(salt: String) -> Result<Self, Self::Error> {
        if salt.len() < 24 {
            bail!("string needs to be at least 24 characters")
        }

        Ok(SignalingSalt(salt))
    }
}

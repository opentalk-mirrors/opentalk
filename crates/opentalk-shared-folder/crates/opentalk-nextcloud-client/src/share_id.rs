// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    derive_more::From,
    derive_more::Into,
    derive_more::FromStr,
    derive_more::AsRef,
    derive_more::Display,
)]
pub struct ShareId(String);

impl ShareId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

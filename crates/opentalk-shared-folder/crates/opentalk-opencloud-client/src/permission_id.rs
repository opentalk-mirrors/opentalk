// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    derive_more::From,
    derive_more::Into,
    derive_more::FromStr,
    derive_more::AsRef,
    derive_more::Display,
)]
pub struct PermissionId(String);

impl PermissionId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

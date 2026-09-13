// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::From;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Debug, Clone, Serialize, Deserialize, From)]
/// Marker which is used for OIDC back-channel logout
pub struct LogoutMarker(u64);

impl LogoutMarker {
    /// Retreive the marker value
    pub fn value(&self) -> u64 {
        self.0
    }
    /// Increment the marker
    pub fn increment(&mut self) {
        self.0 += 1;
    }
}

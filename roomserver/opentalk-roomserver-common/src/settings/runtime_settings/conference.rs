// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use crate::settings::{settings_file, signaling_salt::SignalingSalt};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Conference {
    pub signaling_salt: SignalingSalt,
}

impl From<settings_file::conference::Conference> for Conference {
    fn from(value: settings_file::conference::Conference) -> Self {
        Self {
            signaling_salt: value.signaling_salt,
        }
    }
}

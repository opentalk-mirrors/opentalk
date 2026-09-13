// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use serde::Deserialize;

use crate::settings::signaling_salt::SignalingSalt;

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct Conference {
    #[serde(default)]
    pub signaling_salt: SignalingSalt,
}

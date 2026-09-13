// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use serde::Deserialize;

use crate::settings::settings_file::{
    conference::Conference, defaults::Defaults, reports::Reports,
};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Task {
    #[serde(default)]
    pub(crate) conference: Conference,

    #[serde(default)]
    pub(crate) defaults: Option<Defaults>,

    #[serde(default)]
    pub(crate) reports: Option<Reports>,
}

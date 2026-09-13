// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use crate::settings::{Conference, Defaults, Reports, settings_file};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Task {
    pub conference: Conference,

    pub defaults: Option<Defaults>,

    pub reports: Reports,
}

impl From<settings_file::task::Task> for Task {
    fn from(
        settings_file::task::Task {
            conference,
            defaults,
            reports,
        }: settings_file::task::Task,
    ) -> Self {
        Self {
            conference: conference.into(),
            defaults: defaults.map(Into::into),
            reports: reports.unwrap_or_default().into(),
        }
    }
}

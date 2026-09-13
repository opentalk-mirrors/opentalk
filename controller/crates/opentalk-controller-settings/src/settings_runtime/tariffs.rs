// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::TariffAssignment;
use crate::settings_file;

/// Tariffs configuration.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Tariffs {
    /// The tariff assignment.
    pub assignment: TariffAssignment,
}

impl From<settings_file::Tariffs> for Tariffs {
    fn from(settings_file::Tariffs { assignment }: settings_file::Tariffs) -> Self {
        Self {
            assignment: assignment.map(Into::into).unwrap_or_default(),
        }
    }
}

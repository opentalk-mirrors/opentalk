// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use crate::settings_file;

/// Mapping of external tariff status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TariffStatusMapping {
    /// The name of the tariff to choose as the downgraded OpenTalk tariff.
    pub downgraded_tariff_name: String,

    /// The set of tariff status values that are mapped to the default tariff state.
    pub default: BTreeSet<String>,

    /// The set of tariff status values that are mapped to the paid tariff state.
    pub paid: BTreeSet<String>,

    /// The set of tariff status values that are mapped to the downgraded tariff state.
    pub downgraded: BTreeSet<String>,
}

impl From<settings_file::TariffStatusMapping> for TariffStatusMapping {
    fn from(
        settings_file::TariffStatusMapping {
            downgraded_tariff_name,
            default,
            paid,
            downgraded,
        }: settings_file::TariffStatusMapping,
    ) -> Self {
        Self {
            downgraded_tariff_name,
            default,
            paid,
            downgraded,
        }
    }
}

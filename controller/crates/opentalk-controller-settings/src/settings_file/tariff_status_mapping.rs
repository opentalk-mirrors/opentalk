// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct TariffStatusMapping {
    pub downgraded_tariff_name: String,
    pub default: BTreeSet<String>,
    pub paid: BTreeSet<String>,
    pub downgraded: BTreeSet<String>,
}

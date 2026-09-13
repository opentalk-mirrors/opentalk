// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

use super::TariffStatusMapping;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case", tag = "assignment")]
pub(crate) enum TariffAssignment {
    Static {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        static_tariff_name: Option<String>,
    },
    ByExternalTariffId {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        status_mapping: Option<TariffStatusMapping>,
    },
}

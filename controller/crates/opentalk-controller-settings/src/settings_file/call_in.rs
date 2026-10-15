// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub(crate) struct CallIn {
    pub tel: String,
    #[serde(default = "default_mask_unmapped_numbers")]
    pub mask_unmapped_numbers: bool,
    pub default_country_code: phonenumber::country::Id,
}

const fn default_mask_unmapped_numbers() -> bool {
    true
}

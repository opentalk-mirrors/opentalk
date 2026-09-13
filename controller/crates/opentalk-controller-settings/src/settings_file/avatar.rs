// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub(crate) struct Avatar {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub libravatar_url: Option<String>,
}

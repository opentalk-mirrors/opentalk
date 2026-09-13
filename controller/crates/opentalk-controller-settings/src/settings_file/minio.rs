// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub(crate) struct MinIO {
    pub uri: String,
    pub bucket: String,
    pub region: Option<String>,
    pub force_path_style: Option<bool>,
    pub access_key: String,
    pub secret_key: String,
}

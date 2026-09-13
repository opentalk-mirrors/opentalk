// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub(crate) struct Frontend {
    pub base_url: Url,
}

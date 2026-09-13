// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

#[derive(Clone, Default, Debug, PartialEq, Eq, Deserialize)]
pub(crate) struct Endpoints {
    #[serde(default)]
    pub(crate) disable_users_find: Option<bool>,

    #[serde(default)]
    pub(crate) users_find_use_kc: Option<bool>,

    #[serde(default)]
    pub(crate) event_invite_external_email_address: Option<bool>,

    #[serde(default)]
    pub(crate) disallow_custom_display_name: Option<bool>,

    #[serde(default)]
    pub(crate) disable_openapi: Option<bool>,
}

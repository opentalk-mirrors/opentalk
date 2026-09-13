// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::settings_file;

/// Endpoints configuration
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Endpoints {
    /// Allow inviting external email addresses to events.
    pub event_invite_external_email_address: bool,

    /// Disallow selecting a custom display name.
    pub disallow_custom_display_name: bool,

    /// Disable the OpenAPI endpoint.
    pub disable_openapi: bool,
}

impl From<settings_file::Endpoints> for Endpoints {
    fn from(
        settings_file::Endpoints {
            disable_users_find: _,
            users_find_use_kc: _,
            event_invite_external_email_address,
            disallow_custom_display_name,
            disable_openapi,
        }: settings_file::Endpoints,
    ) -> Self {
        Self {
            event_invite_external_email_address: event_invite_external_email_address
                .unwrap_or_default(),
            disallow_custom_display_name: disallow_custom_display_name.unwrap_or_default(),
            disable_openapi: disable_openapi.unwrap_or_default(),
        }
    }
}

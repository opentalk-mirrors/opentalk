// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use icu_locid::LanguageIdentifier;
use opentalk_types_common::time::TimeZone;

/// Relevant info returned from `userinfo` endpoint.
#[derive(Debug)]
#[must_use]
pub struct OpenIdConnectUserInfo {
    /// The users subject identifier, assigned by the OIDC provider
    pub sub: String,
    /// The email address
    pub email: String,
    /// The users firstname
    pub firstname: String,
    /// The last name
    pub lastname: String,
    /// The URL to get the avatar from
    pub avatar_url: Option<String>,
    /// The locale of the user
    pub locale: Option<LanguageIdentifier>,
    /// The timezone of the user
    pub timezone: Option<TimeZone>,
    /// The groups
    pub groups: Vec<String>,
    /// The phone number
    pub phone_number: Option<String>,
    /// The display name
    pub display_name: Option<String>,
    /// The tenant id
    pub tenant_id: Option<String>,
    /// The tariff id
    pub tariff_id: Option<String>,
    /// The tariff status
    pub tariff_status: Option<String>,
}

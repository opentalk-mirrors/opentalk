// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use icu_locid::LanguageIdentifier;
use opentalk_types_common::{
    tariffs::{TariffId, TariffStatus},
    tenants::TenantId,
    time::TimeZone,
    users::{DisplayName, UserTitle},
};

/// The representation of a new user that is intended to be stored in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewUser {
    /// The OIDC sub field of the user.
    pub oidc_sub: String,

    /// The E-Mail address of the user.
    pub email: String,

    /// The title of the user.
    pub title: UserTitle,

    /// The first name of the user.
    pub firstname: String,

    /// The last name of the user.
    pub lastname: String,

    /// The language of the user.
    pub language: Option<LanguageIdentifier>,

    /// The display name of the user.
    pub display_name: DisplayName,

    /// The phone number of the user.
    pub phone: Option<String>,

    /// The id of the tenant to which the user belongs.
    pub tenant_id: TenantId,

    /// The id of the tariff that is assigned to the user.
    pub tariff_id: TariffId,

    /// The status of the tariff that is assigned to the user.
    pub tariff_status: TariffStatus,

    /// The URL to the avatar of the user.
    pub avatar_url: Option<String>,

    /// The time zone of the user.
    pub timezone: Option<TimeZone>,
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use icu_locid::LanguageIdentifier;
use opentalk_types_common::{
    tariffs::{TariffId, TariffStatus},
    time::{TimeZone, Timestamp},
    users::{DisplayName, Theme, UserTitle},
};

/// Representation of an update to a user in the inventory.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct UpdateUser<'a> {
    /// Update the title.
    pub title: Option<&'a UserTitle>,

    /// Update the e-mail address.
    pub email: Option<String>,

    /// Update the first name.
    pub firstname: Option<&'a str>,

    /// Update the last name.
    pub lastname: Option<&'a str>,

    /// Update the phone number.
    pub phone: Option<Option<String>>,

    /// Update the display name.
    pub display_name: Option<&'a DisplayName>,

    /// Update the language.
    pub language: Option<Option<LanguageIdentifier>>,

    /// Update the dashboard theme.
    pub dashboard_theme: Option<Option<&'a Theme>>,

    /// Update the conference theme.
    pub conference_theme: Option<Option<&'a Theme>>,

    // The tenant_id should never be updated!
    //pub tenant_id: Option<TenantId>,
    //
    /// Update the tariff id.
    pub tariff_id: Option<TariffId>,

    /// Update the tariff status.
    pub tariff_status: Option<TariffStatus>,

    /// Update the optional disabled-since timestamp.
    pub disabled_since: Option<Option<Timestamp>>,

    /// Update the avatar url.
    pub avatar_url: Option<Option<&'a str>>,

    /// Update the timezone.
    pub timezone: Option<Option<TimeZone>>,

    /// Update the updated_at timestamp.
    pub updated_at: Timestamp,
}

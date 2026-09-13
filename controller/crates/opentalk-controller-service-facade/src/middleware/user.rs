// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    tariffs::{TariffId, TariffStatus},
    tenants::TenantId,
    users::{DisplayName, Language, Theme, UserId, UserTitle},
};

/// The user that has made a request as provided by the middleware
#[derive(Clone, Debug)]
pub struct RequestUser {
    /// The user id
    pub id: UserId,
    // pub id_serial: SerialUserId,
    // pub oidc_sub: String,
    /// The user's email address
    pub email: String,
    /// The user's title
    pub title: UserTitle,
    /// The user's first name
    pub firstname: String,
    /// The user's last name
    pub lastname: String,
    /// The language the user has chosen
    pub language: Option<Language>,
    /// The user's display name
    pub display_name: DisplayName,
    /// The theme the user uses for the dashboard
    pub dashboard_theme: Option<Theme>,
    /// The theme the user uses for the conference
    pub conference_theme: Option<Theme>,
    // pub phone: Option<String>,
    /// The user's tenant id
    pub tenant_id: TenantId,
    /// The user's tariff id
    pub tariff_id: TariffId,
    /// The current tariff status
    pub tariff_status: TariffStatus,
    // pub disabled_since: Option<DateTime<Utc>>,
    /// The URL to the user's avatar
    pub avatar_url: Option<String>,
}

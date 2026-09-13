// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    tariffs::{TariffId, TariffStatus},
    tenants::TenantId,
    time::{TimeZone, Timestamp},
    users::{DisplayName, Language, Theme, UserId, UserTitle},
    utils::ExampleData,
};

use crate::Event;

/// The representation of a user in the inventory.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct User {
    /// The id of the user.
    pub id: UserId,

    /// The serial id of the user.
    pub id_serial: i64,

    /// The OIDC sub of the user.
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
    pub language: Option<Language>,

    /// The display name of the user.
    pub display_name: DisplayName,

    /// The dashboard theme.
    pub dashboard_theme: Option<Theme>,

    /// The conference theme.
    pub conference_theme: Option<Theme>,

    /// The phone number of the user.
    pub phone: Option<String>,

    /// The id of the tenant to which the user belongs.
    pub tenant_id: TenantId,

    /// The id of the tariff that is assigned to the user.
    pub tariff_id: TariffId,

    /// The status of the tariff that is assigned to the user.
    pub tariff_status: TariffStatus,

    /// Optional disabled-since timestamp.
    pub disabled_since: Option<Timestamp>,

    /// The URL to the avatar of the user.
    pub avatar_url: Option<String>,

    /// The time zone of the user.
    pub timezone: Option<TimeZone>,

    /// The creation timestamp.
    pub created_at: Timestamp,

    /// The updated timestamp.
    pub updated_at: Timestamp,
}

impl User {
    /// Tell whether a user is allowed to edit an event.
    pub fn can_edit(&self, event: &Event) -> bool {
        self.id == event.created_by
    }
}

impl ExampleData for User {
    fn example_data() -> Self {
        use std::str::FromStr;

        Self {
            id: UserId::nil(),
            id_serial: 0,
            oidc_sub: String::from("12345678-90ab-cdef-1234-567890abcdef"),
            email: String::from("email@mail.com"),
            title: UserTitle::from_str("M.Sc.").unwrap(),
            firstname: String::from("John"),
            lastname: String::from("Doe"),
            language: Some("en".parse().expect("valid language")),
            display_name: DisplayName::from_str("John Doe").unwrap(),
            dashboard_theme: Some(Theme::Light),
            conference_theme: Some(Theme::Light),
            phone: Some(String::from("+1234567890")),
            tenant_id: TenantId::nil(),
            tariff_id: TariffId::nil(),
            tariff_status: TariffStatus::Paid,
            disabled_since: None,
            avatar_url: Some(String::from("www.avatar.com/avatar.png")),
            timezone: Some(TimeZone::utc()),
            created_at: Timestamp::unix_epoch(),
            updated_at: Timestamp::unix_epoch(),
        }
    }
}

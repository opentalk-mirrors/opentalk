// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use opentalk_inventory as inventory;
use opentalk_types_common::{
    tariffs::{TariffId, TariffStatus},
    time::TimeZone,
    users::{DisplayName, Theme, UserTitle},
};

use crate::{schema::users, tables::users::LanguageIdentifier};

/// Diesel user struct for updates
///
/// Is used in update queries. None fields will be ignored on update queries
#[derive(Default, AsChangeset)]
#[diesel(table_name = users)]
pub struct UpdateUser<'a> {
    pub title: Option<&'a UserTitle>,
    pub email: Option<String>,
    pub firstname: Option<&'a str>,
    pub lastname: Option<&'a str>,
    pub phone: Option<Option<String>>,
    pub display_name: Option<&'a DisplayName>,
    pub language: Option<Option<LanguageIdentifier>>,
    pub dashboard_theme: Option<Option<Theme>>,
    pub conference_theme: Option<Option<Theme>>,
    // The tenant_id should never be updated!
    //pub tenant_id: Option<TenantId>,
    pub tariff_id: Option<TariffId>,
    pub tariff_status: Option<TariffStatus>,
    pub disabled_since: Option<Option<DateTime<Utc>>>,
    pub avatar_url: Option<Option<&'a str>>,
    pub timezone: Option<Option<TimeZone>>,
    pub updated_at: DateTime<Utc>,
}

impl<'a> From<inventory::UpdateUser<'a>> for UpdateUser<'a> {
    fn from(
        inventory::UpdateUser {
            title,
            email,
            firstname,
            lastname,
            phone,
            display_name,
            language,
            dashboard_theme,
            conference_theme,
            tariff_id,
            tariff_status,
            disabled_since,
            avatar_url,
            timezone,
            updated_at,
        }: inventory::UpdateUser<'a>,
    ) -> UpdateUser<'a> {
        UpdateUser {
            title,
            email,
            firstname,
            lastname,
            phone,
            display_name,
            language: language.map(|l| l.map(Into::into)),
            dashboard_theme: dashboard_theme.map(|t| t.copied()),
            conference_theme: conference_theme.map(|t| t.copied()),
            tariff_id,
            tariff_status,
            disabled_since: disabled_since.map(|d| d.map(Into::into)),
            avatar_url,
            timezone,
            updated_at: updated_at.into(),
        }
    }
}

impl UpdateUser<'_> {
    pub fn is_empty(&self) -> bool {
        matches!(
            self,
            Self {
                title: None,
                email: None,
                firstname: None,
                lastname: None,
                phone: None,
                display_name: None,
                language: None,
                dashboard_theme: None,
                conference_theme: None,
                tariff_id: None,
                tariff_status: None,
                disabled_since: None,
                avatar_url: None,
                timezone: None,
                updated_at: _
            }
        )
    }
}

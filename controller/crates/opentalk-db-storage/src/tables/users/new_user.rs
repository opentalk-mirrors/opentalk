// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::Insertable;
use opentalk_inventory as inventory;
use opentalk_types_common::{
    tariffs::{TariffId, TariffStatus},
    tenants::TenantId,
    time::TimeZone,
    users::{DisplayName, UserTitle},
};

use crate::{schema::users, tables::users::LanguageIdentifier};

/// Diesel insertable user struct
///
/// Represents fields that have to be provided on user insertion.
#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub oidc_sub: String,
    pub email: String,
    pub title: UserTitle,
    pub firstname: String,
    pub lastname: String,
    pub language: Option<LanguageIdentifier>,
    pub display_name: DisplayName,
    pub phone: Option<String>,
    pub tenant_id: TenantId,
    pub tariff_id: TariffId,
    pub tariff_status: TariffStatus,
    pub avatar_url: Option<String>,
    pub timezone: Option<TimeZone>,
}

impl From<inventory::NewUser> for NewUser {
    fn from(
        inventory::NewUser {
            oidc_sub,
            email,
            title,
            firstname,
            lastname,
            language,
            display_name,
            phone,
            tenant_id,
            tariff_id,
            tariff_status,
            avatar_url,
            timezone,
        }: inventory::NewUser,
    ) -> Self {
        Self {
            oidc_sub,
            email,
            title,
            firstname,
            lastname,
            language: language.map(Into::into),
            display_name,
            phone,
            tenant_id,
            tariff_id,
            tariff_status,
            avatar_url,
            timezone,
        }
    }
}

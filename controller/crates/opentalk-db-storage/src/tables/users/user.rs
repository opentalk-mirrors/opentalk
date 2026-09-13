// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::fmt;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use opentalk_inventory as inventory;
use opentalk_types_common::{
    tariffs::{TariffId, TariffStatus},
    tenants::TenantId,
    time::TimeZone,
    users::{DisplayName, Language, Theme, UserId, UserTitle},
};

use crate::{
    schema::users,
    tables::users::{LanguageIdentifier, SerialUserId},
};

/// Diesel user struct
///
/// Is used as a result in various queries. Represents a user column
#[derive(Clone, Queryable, Identifiable, PartialEq, Eq)]
#[diesel(table_name=users)]
pub struct User {
    pub id: UserId,
    pub id_serial: SerialUserId,
    pub oidc_sub: String,
    pub email: String,
    pub title: UserTitle,
    pub firstname: String,
    pub lastname: String,
    pub language: Option<LanguageIdentifier>,
    pub display_name: DisplayName,
    pub dashboard_theme: Option<Theme>,
    pub conference_theme: Option<Theme>,
    pub phone: Option<String>,
    pub tenant_id: TenantId,
    pub tariff_id: TariffId,
    pub tariff_status: TariffStatus,
    pub disabled_since: Option<DateTime<Utc>>,
    pub avatar_url: Option<String>,
    pub timezone: Option<TimeZone>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_authenticated_at: Option<DateTime<Utc>>,
}

impl From<User> for inventory::User {
    fn from(
        User {
            id,
            id_serial,
            oidc_sub,
            email,
            title,
            firstname,
            lastname,
            language,
            display_name,
            dashboard_theme,
            conference_theme,
            phone,
            tenant_id,
            tariff_id,
            tariff_status,
            disabled_since,
            avatar_url,
            timezone,
            created_at,
            updated_at,
            last_authenticated_at: _,
        }: User,
    ) -> Self {
        Self {
            id,
            id_serial: id_serial.into(),
            oidc_sub,
            email,
            title,
            firstname,
            lastname,
            language: language.map(Into::into),
            display_name,
            dashboard_theme,
            conference_theme,
            phone,
            tenant_id,
            tariff_id,
            tariff_status,
            disabled_since: disabled_since.map(Into::into),
            avatar_url,
            timezone,
            created_at: created_at.into(),
            updated_at: updated_at.into(),
        }
    }
}

impl From<inventory::User> for User {
    fn from(
        inventory::User {
            id,
            id_serial,
            oidc_sub,
            email,
            title,
            firstname,
            lastname,
            language,
            display_name,
            dashboard_theme,
            conference_theme,
            phone,
            tenant_id,
            tariff_id,
            tariff_status,
            disabled_since,
            avatar_url,
            timezone,
            created_at,
            updated_at,
        }: inventory::User,
    ) -> Self {
        Self {
            id,
            id_serial: id_serial.into(),
            oidc_sub,
            email,
            title,
            firstname,
            lastname,
            language: language.map(Into::into),
            last_authenticated_at: None,
            display_name,
            dashboard_theme,
            conference_theme,
            phone,
            tenant_id,
            tariff_id,
            tariff_status,
            disabled_since: disabled_since.map(Into::into),
            avatar_url,
            timezone,
            created_at: created_at.into(),
            updated_at: updated_at.into(),
        }
    }
}

impl fmt::Debug for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("first_name", &self.firstname)
            .field("last_name", &self.lastname)
            .finish()
    }
}

impl User {
    pub fn into_mail_worker_registered_user(
        user: User,
        default_user_language: Language,
    ) -> opentalk_mail_worker_protocol::v1::RegisteredUser {
        opentalk_mail_worker_protocol::v1::RegisteredUser {
            email: user.email.into(),
            title: user.title,
            first_name: user.firstname,
            last_name: user.lastname,
            language: user
                .language
                .map(Into::into)
                .unwrap_or(default_user_language),
        }
    }
}

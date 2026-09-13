// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::DateTime;
use rkyv::{Archive, Deserialize, Serialize};
use snafu::{OptionExt as _, ResultExt as _};
use uuid::Uuid;

use super::{
    DecodeFromCacheError, TariffStatus,
    decode_from_cache_error::{ConvertSnafu, ParseSnafu},
};

/// (De)Serializable version of [`opentalk_inventory::User`] so it can be externally cached
#[derive(Serialize, Deserialize, Archive, Debug, Clone, PartialEq, Eq)]
pub struct User {
    id: Uuid,
    id_serial: i64,
    oidc_sub: String,
    email: String,
    title: String,
    firstname: String,
    lastname: String,
    language: Option<String>,
    display_name: String,
    dashboard_theme: Option<String>,
    conference_theme: Option<String>,
    phone: Option<String>,
    tenant_id: Uuid,
    tariff_id: Uuid,
    tariff_status: TariffStatus,
    disabled_since: Option<i64>,
    avatar_url: Option<String>,
    timezone: Option<String>,
    created_at: i64,
    updated_at: i64,
}

impl User {
    pub fn oidc_sub(&self) -> &String {
        &self.oidc_sub
    }
}

impl From<opentalk_inventory::User> for User {
    fn from(
        opentalk_inventory::User {
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
        }: opentalk_inventory::User,
    ) -> Self {
        Self {
            id: id.into(),
            id_serial,
            oidc_sub,
            email,
            title: title.to_string(),
            firstname,
            lastname,
            language: language.map(|v| v.to_string()),
            display_name: display_name.to_string(),
            dashboard_theme: dashboard_theme.map(|v| v.to_string()),
            conference_theme: conference_theme.map(|v| v.to_string()),
            phone,
            tenant_id: tenant_id.into(),
            tariff_id: tariff_id.into(),
            tariff_status: tariff_status.into(),
            disabled_since: disabled_since.map(|v| DateTime::from(v).timestamp_millis()),
            avatar_url,
            timezone: timezone.map(|v| v.to_string()),
            created_at: DateTime::from(created_at).timestamp_millis(),
            updated_at: DateTime::from(updated_at).timestamp_millis(),
        }
    }
}

impl TryFrom<User> for opentalk_inventory::User {
    type Error = DecodeFromCacheError;

    fn try_from(
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
        }: User,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: id.into(),
            id_serial,
            oidc_sub,
            email,
            title: title
                .parse()
                .map_err(Into::into)
                .context(ParseSnafu { field: "title" })?,
            firstname,
            lastname,
            language: language.map(|v| v.parse()).transpose().map_err(
                |e: icu_locid::ParserError| {
                    ConvertSnafu {
                        field: "language",
                        message: e.to_string(),
                    }
                    .build()
                },
            )?,
            display_name: display_name
                .parse()
                .map_err(Into::into)
                .context(ParseSnafu {
                    field: "display_name",
                })?,
            dashboard_theme: dashboard_theme
                .map(|v| v.parse())
                .transpose()
                .map_err(Into::into)
                .context(ParseSnafu {
                    field: "dashboard_theme",
                })?,
            conference_theme: conference_theme
                .map(|v| v.parse())
                .transpose()
                .map_err(Into::into)
                .context(ParseSnafu {
                    field: "conference_theme",
                })?,
            phone,
            tenant_id: tenant_id.into(),
            tariff_id: tariff_id.into(),
            tariff_status: tariff_status.into(),
            disabled_since: disabled_since
                .map(|v| {
                    DateTime::from_timestamp_millis(v).context(ConvertSnafu {
                        field: "disabled_since",
                        message: format!("Not a valid milliseconds value: {v}"),
                    })
                })
                .transpose()?
                .map(Into::into),
            avatar_url,
            timezone: timezone
                .map(|v| {
                    v.parse().map_err(|e| {
                        ConvertSnafu {
                            field: "timezone",
                            message: format!("Not a valid timezone {v:?}: {e}"),
                        }
                        .build()
                    })
                })
                .transpose()?,
            created_at: DateTime::from_timestamp_millis(created_at)
                .context(ConvertSnafu {
                    field: "created_at",
                    message: format!("Not a valid milliseconds value: {created_at}"),
                })?
                .into(),
            updated_at: DateTime::from_timestamp_millis(updated_at)
                .context(ConvertSnafu {
                    field: "updated_at",
                    message: format!("Not a valid milliseconds value: {updated_at}"),
                })?
                .into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use opentalk_types_common::{tariffs::TariffId, tenants::TenantId, users::UserId};
    use rkyv::{from_bytes, to_bytes};

    use super::User;

    #[test]
    fn roundtrip() {
        let raw = opentalk_inventory::User {
            id: UserId::from_u128(0x123456),
            id_serial: 1337,
            oidc_sub: "theo_user".to_string(),
            email: "the_user@example.com".to_string(),
            title: "Dr.".parse().unwrap(),
            firstname: "Theo".to_string(),
            lastname: "User".to_string(),
            language: Some("de".parse().unwrap()),
            display_name: "Dr. Theo User".parse().unwrap(),
            dashboard_theme: Some("dark".parse().unwrap()),
            conference_theme: Some("system".parse().unwrap()),
            phone: Some("+49-555-555".to_string()),
            tenant_id: TenantId::from_u128(0x333333),
            tariff_id: TariffId::from_u128(0x555555),
            tariff_status: opentalk_types_common::tariffs::TariffStatus::Paid,
            disabled_since: Some("2026-01-10T11:22:33Z".parse().unwrap()),
            avatar_url: Some("https://myavatar.example.com/0x123456".to_string()),
            timezone: Some("Europe/Berlin".parse().unwrap()),
            created_at: "2025-02-06T09:16:47Z".parse().unwrap(),
            updated_at: "2026-01-02T22:10:10Z".parse().unwrap(),
        };

        let encoded = to_bytes::<rkyv::rancor::Error>(&User::from(raw.clone())).unwrap();
        let decoded = from_bytes::<User, rkyv::rancor::Error>(&encoded)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(raw, decoded);
    }
}

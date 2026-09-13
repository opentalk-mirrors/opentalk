// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Display, From, FromStr, Into};
use opentalk_diesel_newtype::DieselNewtype;
use opentalk_inventory as inventory;
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

#[derive(
    AsRef,
    Display,
    From,
    FromStr,
    Into,
    Serialize,
    Deserialize,
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    DieselNewtype,
    AsExpression,
    FromSqlRow,
    ToRedisArgs,
    FromRedisValue,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
#[to_redis_args(fmt = "{0}")]
#[from_redis_value(FromStr)]
pub struct OidcTenantId(String);

impl From<&str> for OidcTenantId {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}

impl From<OidcTenantId> for inventory::OidcTenantId {
    fn from(OidcTenantId(value): OidcTenantId) -> Self {
        Self::from(value)
    }
}

impl From<inventory::OidcTenantId> for OidcTenantId {
    fn from(value: inventory::OidcTenantId) -> Self {
        Self(value.into())
    }
}

impl From<&inventory::OidcTenantId> for OidcTenantId {
    fn from(value: &inventory::OidcTenantId) -> Self {
        value.as_str().into()
    }
}

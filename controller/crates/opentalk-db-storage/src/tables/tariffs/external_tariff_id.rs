// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use core::fmt::Debug;

use derive_more::{AsRef, Display, From, FromStr, Into};
use opentalk_diesel_newtype::DieselNewtype;
use opentalk_inventory as inventory;
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
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    AsExpression,
    FromSqlRow,
    DieselNewtype,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct ExternalTariffId(String);

impl From<ExternalTariffId> for inventory::ExternalTariffId {
    fn from(ExternalTariffId(value): ExternalTariffId) -> Self {
        Self::from(value)
    }
}

impl From<inventory::ExternalTariffId> for ExternalTariffId {
    fn from(value: inventory::ExternalTariffId) -> Self {
        Self(value.into())
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

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
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    AsExpression,
    FromSqlRow,
    DieselNewtype,
)]
#[diesel(sql_type = diesel::sql_types::BigInt)]
pub struct SerialJobId(i64);

impl From<SerialJobId> for inventory::JobId {
    fn from(SerialJobId(value): SerialJobId) -> Self {
        Self::from(value)
    }
}

impl From<inventory::JobId> for SerialJobId {
    fn from(value: inventory::JobId) -> Self {
        Self(value.into())
    }
}

impl From<SerialJobId> for inventory::JobExecutionId {
    fn from(SerialJobId(value): SerialJobId) -> Self {
        Self::from(value)
    }
}

impl From<inventory::JobExecutionId> for SerialJobId {
    fn from(value: inventory::JobExecutionId) -> Self {
        Self(value.into())
    }
}

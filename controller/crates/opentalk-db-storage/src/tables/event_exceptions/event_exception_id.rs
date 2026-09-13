// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Display, From, FromStr, Into};
use opentalk_diesel_newtype::DieselNewtype;
use opentalk_inventory as inventory;
use serde::{Deserialize, Serialize};

#[derive(
    AsExpression,
    AsRef,
    Clone,
    Copy,
    Debug,
    Deserialize,
    DieselNewtype,
    Display,
    Eq,
    From,
    FromSqlRow,
    FromStr,
    Hash,
    Into,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[diesel(sql_type = diesel::sql_types::Uuid)]
pub struct EventExceptionId(uuid::Uuid);

impl From<inventory::EventExceptionId> for EventExceptionId {
    fn from(value: inventory::EventExceptionId) -> Self {
        Self::from(uuid::Uuid::from(value))
    }
}

impl From<EventExceptionId> for inventory::EventExceptionId {
    fn from(EventExceptionId(id): EventExceptionId) -> Self {
        Self::from(id)
    }
}

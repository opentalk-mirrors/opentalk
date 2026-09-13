// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Display, From, FromStr, Into};
use opentalk_diesel_newtype::DieselNewtype;
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
#[diesel(sql_type = diesel::sql_types::BigInt)]
pub struct EventSerialId(i64);

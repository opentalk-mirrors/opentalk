// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::{AsRef, Display, From, FromStr, Into};
use opentalk_diesel_newtype::DieselNewtype;
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
    DieselNewtype,
    AsExpression,
    FromSqlRow,
)]
#[diesel(sql_type = diesel::sql_types::BigInt)]
pub struct SerialGroupId(i64);

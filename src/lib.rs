// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::{AsExpression, TypedExpressionType},
    pg::Pg,
    sql_types::{SingleValue, SqlType},
};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

#[doc(hidden)]
pub mod __exports {
    pub use diesel;
}

pub use opentalk_diesel_newtype_impl::DieselNewtype;

pub trait DieselNewtype<T>:
    Debug
    + Display
    + Clone
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
    + Hash
    + Serialize
    + DeserializeOwned
    + AsExpression<T>
    + FromSqlRow<T, Pg>
where
    T: SqlType + TypedExpressionType + SingleValue,
    Self: FromSql<T, Pg>,
{
}

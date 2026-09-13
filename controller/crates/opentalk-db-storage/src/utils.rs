// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{fmt::Debug, io::Write};

use async_stream::stream;
use diesel::{
    QueryResult,
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    serialize::{IsNull, ToSql},
    sql_types,
};
use futures_core::Stream;
use opentalk_database::DatabaseError;
use serde::{Deserialize, Serialize};

/// JSONB Wrapper for any type implementing the serde `Serialize` or `Deserialize` trait
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, FromSqlRow, AsExpression, PartialEq, Eq,
)]
#[diesel(sql_type = sql_types::Jsonb)]
pub struct Jsonb<T>(pub T);

impl<T: for<'de> Deserialize<'de>> FromSql<sql_types::Jsonb, Pg> for Jsonb<T> {
    fn from_sql(value: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let bytes = value.as_bytes();
        if bytes[0] != 1 {
            return Err("Unsupported JSONB encoding version".into());
        }
        serde_json::from_slice(&bytes[1..])
            .map(Self)
            .map_err(|_| "Invalid Json".into())
    }
}

impl<T: Serialize + Debug> ToSql<sql_types::Jsonb, Pg> for Jsonb<T> {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        out.write_all(&[1])?;
        serde_json::to_writer(out, &self.0)
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

pub(super) fn convert_diesel_query_results<S: Stream<Item = QueryResult<T>>, T>(
    input: S,
) -> impl Stream<Item = opentalk_database::Result<T>> {
    stream! {
        for await value in input {
            match value {
                Ok(value) => { yield Ok(value); }
                Err(error) => { yield Err(DatabaseError::DieselError {source: error}); }
            }
        }
    }
}

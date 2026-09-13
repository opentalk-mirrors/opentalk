// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use byteorder::{NetworkEndian, WriteBytesExt as _};
use diesel::{
    backend::Backend,
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, IsNull, Output, ToSql},
};
use snafu::Snafu;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    diesel::expression::AsExpression,
    diesel::deserialize::FromSqlRow,
    derive_more::From,
    derive_more::Into,
)]
#[diesel(sql_type = diesel::sql_types::BigInt)]
pub struct Duration(pub std::time::Duration);

#[derive(Debug, Snafu)]
pub enum FromLanguageIdentifierError {
    #[snafu(display("Invalid language identifier {found:?}: {message}"))]
    InvalidLanguageIdentifier { found: String, message: String },
}

impl ToSql<diesel::sql_types::BigInt, Pg> for Duration {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let unsigned = self.0.as_secs();
        let signed: i64 = unsigned.try_into()?;
        out.write_i64::<NetworkEndian>(signed)
            .map(|_| IsNull::No)
            .map_err(|e| Box::new(e) as Box<_>)
    }
}

impl FromSql<diesel::sql_types::BigInt, Pg> for Duration {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let signed = i64::from_sql(bytes)?;
        let unsigned = signed.try_into()?;

        Ok(Self(std::time::Duration::from_secs(unsigned)))
    }
}

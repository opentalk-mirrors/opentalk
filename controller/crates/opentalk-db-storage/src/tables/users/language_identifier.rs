// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::io::Write as _;

use diesel::{
    backend::Backend,
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, Output, ToSql},
};
use opentalk_types_common::users::Language;
use snafu::Snafu;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    diesel::expression::AsExpression,
    diesel::deserialize::FromSqlRow,
    derive_more::From,
    derive_more::FromStr,
    derive_more::Into,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct LanguageIdentifier(pub icu_locid::LanguageIdentifier);

#[derive(Debug, Snafu)]
pub enum FromLanguageIdentifierError {
    #[snafu(display("Invalid language identifier {found:?}: {message}"))]
    InvalidLanguageIdentifier { found: String, message: String },
}

impl ToSql<diesel::sql_types::Text, Pg> for LanguageIdentifier {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        write!(out, "{}", self.0)?;
        Ok(diesel::serialize::IsNull::No)
    }
}

impl FromSql<diesel::sql_types::Text, Pg> for LanguageIdentifier {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let s = std::str::from_utf8(bytes.as_bytes())?;
        let id = s.parse().map_err(|e| format!("{e}"))?;

        Ok(Self(id))
    }
}

impl From<Language> for LanguageIdentifier {
    fn from(value: Language) -> Self {
        Self(value.into())
    }
}

impl From<LanguageIdentifier> for Language {
    fn from(LanguageIdentifier(value): LanguageIdentifier) -> Self {
        value.into()
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "typescript",
    derive(ts_rs::TS),
    ts(export_to = "common-identifiers/")
)]
pub(crate) struct Identifier(std::borrow::Cow<'static, str>);

impl Identifier {
    pub(crate) const fn new_borrowed(s: &'static str) -> Self {
        Self(std::borrow::Cow::Borrowed(s))
    }

    pub(crate) const fn new_owned(s: String) -> Self {
        Self(std::borrow::Cow::Owned(s))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for Identifier {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}

impl From<&'static str> for Identifier {
    fn from(value: &'static str) -> Self {
        Self(value.into())
    }
}

impl FromStr for Identifier {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.to_string().into())
    }
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.as_str().eq(other.as_str())
    }
}

impl Eq for Identifier {}

impl PartialOrd for Identifier {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Identifier {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl std::hash::Hash for Identifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

#[cfg(feature = "diesel")]
mod diesel_traits {
    use std::{io::Write, str::from_utf8};

    use diesel::{
        backend::Backend,
        deserialize::{self, FromSql},
        pg::Pg,
        serialize::{self, IsNull, Output, ToSql},
    };

    use super::Identifier;

    impl ToSql<diesel::sql_types::Text, Pg> for Identifier {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
            write!(out, "{}", self.as_str())?;
            Ok(IsNull::No)
        }
    }

    impl FromSql<diesel::sql_types::Text, Pg> for Identifier {
        fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
            Ok(Self(std::borrow::Cow::Owned(
                from_utf8(bytes.as_bytes())?.to_string(),
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Identifier;

    #[test]
    fn from_str() {
        let parsed: Identifier = "hello"
            .parse()
            .expect("value must be parsable as Identifier");
        assert_eq!(parsed, Identifier("hello".to_string().into()));
    }
}

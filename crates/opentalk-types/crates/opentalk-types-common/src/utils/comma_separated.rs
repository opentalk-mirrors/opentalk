// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! (De)Serialize comma separated values with `#[serde(with = "comma_separated")]`

use std::{fmt, marker::PhantomData, str::FromStr};

use itertools::Itertools;
use serde::{Deserializer, Serializer};

/// Helper function to deserialize comma-separated values
pub fn deserialize<'de, V, T, D>(deserializer: D) -> Result<V, D::Error>
where
    V: FromIterator<T>,
    T: FromStr,
    T::Err: fmt::Display,
    D: Deserializer<'de>,
{
    struct CommaSeparated<V, T>(PhantomData<(T, V)>);

    impl<V, T> serde::de::Visitor<'_> for CommaSeparated<V, T>
    where
        V: FromIterator<T>,
        T: FromStr,
        T::Err: fmt::Display,
    {
        type Value = V;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string containing comma-separated elements")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let iter = s.split(',').map(FromStr::from_str);
            iter.collect::<Result<_, _>>()
                .map_err(serde::de::Error::custom)
        }
    }

    let visitor = CommaSeparated(PhantomData);
    deserializer.deserialize_str(visitor)
}

/// Helper function to serialize comma-separated values
pub fn serialize<S, T>(v: &[T], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: fmt::Display,
{
    serializer.serialize_str(&v.iter().map(|v| v.to_string()).join(","))
}

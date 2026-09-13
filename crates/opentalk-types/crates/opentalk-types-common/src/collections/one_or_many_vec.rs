// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! A module providing the [`OneOrManyVec`] type. As a bonus, this module can
//! be used for (de-)serializing a [`Vec`] through the
//! [`OneOrManyVec`] type in a `#[serde(with = "opentalk_types_common::collections::one_or_many_vec")]`
//! attribute. The same is possible for an [`Option<Vec>`] using
//! `#[serde(with = "opentalk_types_common::collections::one_or_many_vec_option")]`

/// A container that either contains a single value or a set of zero or more values.
///
/// This can be used to deserialize JSON data that contains either a single
/// instance or a list of values.
///
/// ```
/// # #[cfg(feature = "serde")] {
/// # use opentalk_types_common::collections::OneOrManyVec;
/// # use serde::{Deserialize, Serialize};
/// # use serde_json::json;
///
/// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// struct Choice {
///     selected: OneOrManyVec<u32>,
/// }
///
/// let single = json!({
///     "selected": 42
/// });
///
/// assert_eq!(
///     serde_json::from_value::<Choice>(single).unwrap(),
///     Choice { selected: OneOrManyVec::One(42u32) }
/// );
///
/// let multiple = json!({
///     "selected": [ 23, 42, 99 ]
/// });
///
/// assert_eq!(
///     serde_json::from_value::<Choice>(multiple).unwrap(),
///     Choice { selected: OneOrManyVec::Many(vec![23, 42, 99]) }
/// );
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(untagged)
)]
pub enum OneOrManyVec<T> {
    /// A single entry of T (typically deserialized from single JSON item)
    One(T),

    /// Multiple entries of T (typically deserialized from a JSON list)
    Many(Vec<T>),
}

impl<T> From<OneOrManyVec<T>> for Vec<T> {
    fn from(value: OneOrManyVec<T>) -> Self {
        match value {
            OneOrManyVec::One(value) => vec![value],
            OneOrManyVec::Many(value) => value,
        }
    }
}

impl<T> From<Vec<T>> for OneOrManyVec<T> {
    fn from(value: Vec<T>) -> Self {
        Self::Many(value)
    }
}

impl<T> From<T> for OneOrManyVec<T> {
    fn from(value: T) -> Self {
        Self::One(value)
    }
}

/// serde deserialize function, needed in order to use this module in the
/// `#[serde(with = "opentalk_types_common::collections::one_or_many_vec")]` attribute.
#[cfg(feature = "serde")]
pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let one_or_many: OneOrManyVec<T> = serde::Deserialize::deserialize(deserializer)?;
    Ok(one_or_many.into())
}

/// serde serialize function, needed in order to use this module in the
/// `#[serde(with = "opentalk_types_common::collections::one_or_many_vec")]` attribute.
#[cfg(feature = "serde")]
pub fn serialize<S, T>(value: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: serde::Serialize,
{
    use serde::Serialize as _;

    value.serialize(serializer)
}

/// module for usage in a
/// `#[serde(with = "opentalk_types_common::collections::one_or_many_vec_option")]` attribute.
#[cfg(feature = "serde")]
pub mod one_or_many_vec_option {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::OneOrManyVec;

    /// serde deserialize function, needed in order to use this module in the
    /// `#[serde(with = "opentalk_types_common::collections::one_or_many_vec_option")]` attribute.
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        let option: Option<OneOrManyVec<T>> = Deserialize::deserialize(deserializer)?;
        Ok(option.map(Vec::from))
    }

    /// serde serialize function, needed in order to use this module in the
    /// `#[serde(with = "opentalk_types_common::collections::one_or_many_vec_option")]` attribute.
    pub fn serialize<S, T>(value: &Option<Vec<T>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        value.serialize(serializer)
    }
}

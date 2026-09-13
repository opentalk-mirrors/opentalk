// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! A module providing the [`OneOrManyBTreeSet`] type. As a bonus, this module can
//! be used for (de-)serializing a [`BTreeSet`] through the
//! [`OneOrManyBTreeSet`] type in a `#[serde(with = "opentalk_types_common::collections::one_or_many_btree_set")]`
//! attribute. The same is possible for an [`Option<BTreeSet>`] using
//! `#[serde(with = "opentalk_types_common::collections::one_or_many_btree_set_option")]`

use std::collections::BTreeSet;

/// A container that either contains a single value or a set of zero or more values.
///
/// This can be used to deserialize JSON data that contains either a single
/// instance or a list of values.
///
/// ```
/// # #[cfg(feature = "serde")] {
/// # use std::collections::BTreeSet;
/// #
/// # use opentalk_types_common::collections::OneOrManyBTreeSet;
/// # use serde::{Deserialize, Serialize};
/// # use serde_json::json;
///
/// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// struct Choice {
///     selected: OneOrManyBTreeSet<u32>,
/// }
///
/// let single = json!({
///     "selected": 42
/// });
///
/// assert_eq!(
///     serde_json::from_value::<Choice>(single).unwrap(),
///     Choice { selected: OneOrManyBTreeSet::One(42u32) }
/// );
///
/// let multiple = json!({
///     "selected": [ 23, 42, 99 ]
/// });
///
/// assert_eq!(
///     serde_json::from_value::<Choice>(multiple).unwrap(),
///     Choice { selected: OneOrManyBTreeSet::Many(BTreeSet::from_iter([23, 42, 99])) }
/// );
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(untagged)
)]
pub enum OneOrManyBTreeSet<T>
where
    T: Ord,
{
    /// A single entry of T (typically deserialized from single JSON item)
    One(T),

    /// Multiple entries of T (typically deserialized from a JSON list)
    Many(BTreeSet<T>),
}

impl<T> From<OneOrManyBTreeSet<T>> for BTreeSet<T>
where
    T: Ord,
{
    fn from(value: OneOrManyBTreeSet<T>) -> Self {
        match value {
            OneOrManyBTreeSet::One(value) => BTreeSet::from_iter([value]),
            OneOrManyBTreeSet::Many(value) => value,
        }
    }
}

impl<T> From<BTreeSet<T>> for OneOrManyBTreeSet<T>
where
    T: Ord,
{
    fn from(value: BTreeSet<T>) -> Self {
        Self::Many(value)
    }
}

impl<T> From<T> for OneOrManyBTreeSet<T>
where
    T: Ord,
{
    fn from(value: T) -> Self {
        Self::One(value)
    }
}

/// serde deserialize function, needed in order to use this module in the
/// `#[serde(with = "opentalk_types_common::collections::one_or_many_btree_set")]` attribute.
#[cfg(feature = "serde")]
pub fn deserialize<'de, D, T>(deserializer: D) -> Result<BTreeSet<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Ord + serde::Deserialize<'de>,
{
    let one_or_many: OneOrManyBTreeSet<T> = serde::Deserialize::deserialize(deserializer)?;
    Ok(one_or_many.into())
}

/// serde serialize function, needed in order to use this module in the
/// `#[serde(with = "opentalk_types_common::collections::one_or_many_btree_set")]` attribute.
#[cfg(feature = "serde")]
pub fn serialize<S, T>(value: &BTreeSet<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: Ord + serde::Serialize,
{
    use serde::Serialize as _;

    value.serialize(serializer)
}

/// module for usage in a
/// `#[serde(with = "opentalk_types_common::collections::one_or_many_btree_set_option")]` attribute.
#[cfg(feature = "serde")]
pub mod one_or_many_btree_set_option {
    use std::collections::BTreeSet;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::OneOrManyBTreeSet;

    /// serde deserialize function, needed in order to use this module in the
    /// `#[serde(with = "opentalk_types_common::collections::one_or_many_btree_set_option")]` attribute.
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<BTreeSet<T>>, D::Error>
    where
        D: Deserializer<'de>,
        T: Ord + Deserialize<'de>,
    {
        let option: Option<OneOrManyBTreeSet<T>> = Deserialize::deserialize(deserializer)?;
        Ok(option.map(BTreeSet::from))
    }

    /// serde serialize function, needed in order to use this module in the
    /// `#[serde(with = "opentalk_types_common::collections::one_or_many_btree_set_option")]` attribute.
    pub fn serialize<S, T>(value: &Option<BTreeSet<T>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Ord + Serialize,
    {
        value.serialize(serializer)
    }
}

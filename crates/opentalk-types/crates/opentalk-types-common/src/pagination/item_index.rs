// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use snafu::{Snafu, ensure};

use crate::utils::ExampleData;

/// The maximum number for an item index to be valid.
pub const ITEM_INDEX_MAX: ItemIndex = ItemIndex(MAX_VALUE);

/// The default value for ItemIndex.
pub const ITEM_INDEX_DEFAULT: ItemIndex = ItemIndex(DEFAULT_VALUE);

const MIN_VALUE: i64 = 0;
const MAX_VALUE: i64 = i64::MAX;
const DEFAULT_VALUE: i64 = 0;
const ZERO_VALUE: i64 = 0;

/// Error when parsing an [`ItemIndex`].
#[derive(Debug, Snafu)]
pub enum TryFromItemIndexError {
    /// Item index is larger than the maximum item index allowed.
    #[snafu(display(
        "Item index is larger than the maximum item index allowed ({item_index_max})."
    ))]
    ValueTooLarge {
        /// The maximum item index
        item_index_max: i64,
    },
    /// Item index is negative.
    #[snafu(display("Item index is negative"))]
    ValueNegative,
}

/// The number of items, e.g. .
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::Display,
    derive_more::AsRef,
    derive_more::Into,
)]
#[cfg_attr(
    feature = "diesel",
    derive(
        opentalk_diesel_newtype::DieselNewtype,
        diesel::expression::AsExpression,
        diesel::deserialize::FromSqlRow
    )
)]
#[cfg_attr(feature="diesel", diesel(sql_type = diesel::sql_types::BigInt))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "i64")
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(ItemIndex::example_data())))]
pub struct ItemIndex(i64);

impl ItemIndex {
    /// The minimum value of the type.
    pub const MIN: Self = Self(MIN_VALUE);

    /// The maximum value of the type.
    pub const MAX: Self = Self(MAX_VALUE);

    /// The default value of the type.
    pub const DEFAULT: Self = Self(DEFAULT_VALUE);

    /// The zero value of the type.
    pub const ZERO: Self = Self(ZERO_VALUE);

    /// Add two [`ItemIndex`] values, saturating.
    pub const fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    /// Subtract an [`ItemIndex`] from another, saturating.
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }

    /// Tell whether the value is the minimum value.
    pub const fn is_min(&self) -> bool {
        self.0 == MIN_VALUE
    }

    /// Tell whether the value is the maximum value.
    pub const fn is_max(&self) -> bool {
        self.0 == MAX_VALUE
    }

    /// Tell whether the value is zero.
    pub const fn is_zero(&self) -> bool {
        self.0 == ZERO_VALUE
    }

    /// Tell whether the value is the default value.
    pub const fn is_default(&self) -> bool {
        self.0 == DEFAULT_VALUE
    }
}

impl TryFrom<u64> for ItemIndex {
    type Error = TryFromItemIndexError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        let value = i64::try_from(value).map_err(|_e| TryFromItemIndexError::ValueTooLarge {
            item_index_max: MAX_VALUE,
        })?;
        Self::try_from(value)
    }
}

impl TryFrom<usize> for ItemIndex {
    type Error = TryFromItemIndexError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let value = i64::try_from(value).map_err(|_e| TryFromItemIndexError::ValueTooLarge {
            item_index_max: MAX_VALUE,
        })?;
        Self::try_from(value)
    }
}

impl TryFrom<i64> for ItemIndex {
    type Error = TryFromItemIndexError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        ensure!(value >= 0, ValueNegativeSnafu);
        Ok(Self(value))
    }
}

impl From<u32> for ItemIndex {
    fn from(value: u32) -> Self {
        Self(value.into())
    }
}

impl From<ItemIndex> for usize {
    fn from(ItemIndex(value): ItemIndex) -> Self {
        usize::try_from(value).unwrap()
    }
}

impl ExampleData for ItemIndex {
    fn example_data() -> Self {
        Self(17)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_matches};

    use super::{ItemIndex, MAX_VALUE, TryFromItemIndexError};

    #[test]
    fn try_from() {
        assert_eq!(ItemIndex::try_from(0usize).unwrap(), ItemIndex::ZERO);
        assert_eq!(ItemIndex::try_from(0i64).unwrap(), ItemIndex::ZERO);
        assert_eq!(ItemIndex::try_from(0u64).unwrap(), ItemIndex::ZERO);

        assert_eq!(ItemIndex::try_from(14usize).unwrap(), ItemIndex(14));
        assert_eq!(ItemIndex::try_from(16i64).unwrap(), ItemIndex(16));
        assert_eq!(ItemIndex::try_from(18u64).unwrap(), ItemIndex(18));

        assert_matches!(
            ItemIndex::try_from(-42i64),
            Err(TryFromItemIndexError::ValueNegative)
        );
        assert_matches!(
            ItemIndex::try_from((i64::MAX as usize) + 1),
            Err(TryFromItemIndexError::ValueTooLarge {
                item_index_max: MAX_VALUE
            })
        );
        assert_matches!(
            ItemIndex::try_from((i64::MAX as u64) + 1),
            Err(TryFromItemIndexError::ValueTooLarge {
                item_index_max: MAX_VALUE
            })
        );
    }

    #[test]
    fn from_u32() {
        assert_eq!(ItemIndex::from(0u32), ItemIndex::ZERO);
        assert_eq!(ItemIndex::from(42u32), ItemIndex(42));
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::ItemIndex;

    #[test]
    fn serialize_default() {
        let example = ItemIndex::default();
        assert_eq!(json!(example), json!(0));
    }

    #[test]
    fn serialize() {
        let example = ItemIndex::from(423);
        assert_eq!(json!(example), json!(423));
    }

    #[test]
    fn deserialize_default() {
        let example = ItemIndex::default();
        assert_eq!(example, serde_json::from_value(json!(0)).unwrap());
    }

    #[test]
    fn deserialize() {
        let example = ItemIndex::from(64);
        assert_eq!(example, serde_json::from_value(json!(64)).unwrap());
    }
}

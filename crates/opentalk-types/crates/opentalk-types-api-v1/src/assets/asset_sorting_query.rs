// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{assets::AssetSorting, order::Ordering};

/// Asset sorting query type
///
/// The struct describes the query parameter that can be provided to sort the returned assets.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema, utoipa::IntoParams))]
pub struct AssetSortingQuery {
    /// The optional sorting query parameter
    #[cfg_attr(feature = "serde", serde(default))]
    pub sort: AssetSorting,

    /// The sorting order that should be applied to the collection
    #[cfg_attr(feature = "serde", serde(default))]
    pub order: Ordering,
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use opentalk_types_common::assets::AssetSorting;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn asset_sort_query() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let paging = AssetSortingQuery {
            sort: AssetSorting::CreatedAt,
            order: Ordering::Descending,
        };

        let paging_urlencoded = "sort=created_at&order=descending";

        let serialize_output: String = serde_urlencoded::to_string(paging)?;
        assert_eq!(paging_urlencoded, serialize_output);

        let deserialized = serde_urlencoded::from_str(paging_urlencoded)?;
        assert_eq!(paging, deserialized);

        Ok(())
    }

    #[test]
    fn invalid_asset_sort_query() {
        use serde::de::Error;

        assert_eq!(
            serde_urlencoded::from_str::<AssetSortingQuery>("sort=wrong_field"),
            Err(serde_urlencoded::de::Error::custom(
                "unknown variant `wrong_field`, expected one of `filename`, `size`, `namespace`, `kind`, `created_at`"
            )),
        );
        assert_eq!(
            serde_urlencoded::from_str::<AssetSortingQuery>("order=wrong_order"),
            Err(serde_urlencoded::de::Error::custom(
                "unknown variant `wrong_order`, expected `ascending` or `descending`"
            )),
        );
    }

    #[test]
    fn asset_query_default_values() {
        let default_ordering = AssetSortingQuery::default();

        assert_eq!(
            Ok(default_ordering),
            serde_urlencoded::from_str::<AssetSortingQuery>("")
        );
    }
}

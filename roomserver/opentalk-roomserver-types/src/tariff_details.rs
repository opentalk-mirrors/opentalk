// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use opentalk_types_common::{
    features::ModuleFeatureId,
    tariffs::{QuotaType, TariffId},
    utils::ExampleData,
};
use serde::{Deserialize, Serialize};

/// Tariff information related to a room
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(TariffDetails::example_data())),
)]
pub struct TariffDetails {
    /// The ID of the tariff
    pub id: TariffId,

    /// The name of the tariff
    pub name: String,

    /// The quotas of the tariff
    pub quotas: BTreeMap<QuotaType, u64>,

    /// The amount of used quota from the tariff
    pub used_quota: BTreeMap<QuotaType, u64>,

    /// Disabled module features
    pub disabled_features: BTreeSet<ModuleFeatureId>,
}

impl TariffDetails {
    /// Get a quota value. Returns [`None`] if the quota is not set.
    pub fn quota(&self, quota: &QuotaType) -> Option<u64> {
        self.quotas.get(quota).copied()
    }

    /// Get a used quota value. Returns 0 if the quota is not set.
    pub fn used_quota(&self, quota: &QuotaType) -> u64 {
        self.used_quota.get(quota).copied().unwrap_or(0)
    }
}

impl ExampleData for TariffDetails {
    fn example_data() -> Self {
        Self {
            id: TariffId::nil(),
            name: "Starter tariff".to_string(),
            quotas: BTreeMap::from_iter([(QuotaType::MaxStorage, 50000)]),
            used_quota: BTreeMap::from_iter([(QuotaType::MaxStorage, 20000)]),
            disabled_features: ["recording::record".parse().expect("valid feature id")]
                .into_iter()
                .collect(),
        }
    }
}

#[test]
fn test_tariff_details_example_data() {
    // Test that example data can be created without panicking
    TariffDetails::example_data();
}

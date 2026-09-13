// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use opentalk_types_common::{
    features::{FeatureId, ModuleFeatureId},
    modules::ModuleId,
    tariffs::{QuotaType, TariffId, TariffModuleResource, TariffResource},
    time::Timestamp,
};

/// The representation of a tariff in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tariff {
    /// The id of the tariff.
    pub id: TariffId,

    /// The name of the tariff.
    pub name: String,

    /// The creation timestamp.
    pub created_at: Timestamp,

    /// The update timestamp.
    pub updated_at: Timestamp,

    /// The quotas for the tariff.
    pub quotas: BTreeMap<QuotaType, u64>,

    /// The set of disabled modules in the tariff.
    pub disabled_modules: Vec<Option<ModuleId>>,

    /// The list of disabled features in the tariff.
    pub disabled_features: Vec<Option<ModuleFeatureId>>,
}

impl Tariff {
    /// Get a quota defined in the tariff. Will return [`None`] if the quota is not defined.
    pub fn quota(&self, quota: &QuotaType) -> Option<u64> {
        self.quotas.get(quota).copied()
    }

    /// Get the list of disabled modules for the tariff.
    pub fn disabled_modules(&self) -> BTreeSet<ModuleId> {
        self.disabled_modules.iter().flatten().cloned().collect()
    }

    /// Get the list of disabled features for the tariff.
    pub fn disabled_features(&self) -> BTreeSet<ModuleFeatureId> {
        self.disabled_features.iter().flatten().cloned().collect()
    }

    /// Query if a specific module feature is disabled in the tariff.
    pub fn is_feature_disabled(&self, module_feature: &ModuleFeatureId) -> bool {
        self.disabled_features
            .contains(&Some(module_feature.clone()))
    }

    /// Create a [`TariffResource`] based on the tariff.
    ///
    /// The given disabled features and the set of available modules with their
    /// features is used to calculate the effectively available modules and
    /// features.
    pub fn to_tariff_resource(
        &self,
        disabled_features: impl IntoIterator<Item = ModuleFeatureId>,
        module_features: BTreeMap<ModuleId, impl IntoIterator<Item = FeatureId>>,
    ) -> TariffResource {
        let disabled_modules = self.disabled_modules();

        let disabled_features: BTreeSet<_> = BTreeSet::from_iter(
            self.disabled_features()
                .into_iter()
                .chain(disabled_features),
        );

        let mut modules = BTreeMap::<ModuleId, TariffModuleResource>::new();

        module_features
            .into_iter()
            .for_each(|(module_id, feature_id)| {
                if !disabled_modules.contains(&module_id) {
                    let features: BTreeSet<FeatureId> =
                        BTreeSet::from_iter(feature_id.into_iter().filter(|feature| {
                            !disabled_features.contains(&ModuleFeatureId {
                                module: module_id.clone(),
                                feature: feature.clone(),
                            })
                        }));
                    let module_resource = TariffModuleResource { features };
                    _ = modules.insert(module_id, module_resource);
                }
            });

        TariffResource {
            id: self.id,
            name: self.name.clone(),
            quotas: self.quotas.clone(),
            modules,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn tariff_to_tariff_resource() {
        let tariff = Tariff {
            id: TariffId::nil(),
            name: "test".into(),
            created_at: Default::default(),
            updated_at: Default::default(),
            quotas: Default::default(),
            disabled_modules: vec![
                Some("whiteboard".parse().expect("valid module id")),
                Some("timer".parse().expect("valid module id")),
                Some("media".parse().expect("valid module id")),
                Some("polls".parse().expect("valid module id")),
            ],
            disabled_features: vec![Some(
                "chat::chat_feature_1"
                    .parse()
                    .expect("valid module feature id"),
            )],
        };

        let module_features = BTreeMap::from([
            (
                "chat".parse().expect("valid module id"),
                BTreeSet::from([
                    "chat_feature_1".parse().expect("valid feature id"),
                    "chat_feature_2".parse().expect("valid feature id"),
                ]),
            ),
            ("media".parse().expect("valid moudle id"), BTreeSet::new()),
            ("polls".parse().expect("valid module id"), BTreeSet::new()),
            (
                "whiteboard".parse().expect("valid module id"),
                BTreeSet::new(),
            ),
            ("timer".parse().expect("valid module id"), BTreeSet::new()),
        ]);

        let expected = json!({
            "id": "00000000-0000-0000-0000-000000000000",
            "name": "test",
            "quotas": {},
            "modules": {
                "chat": {
                    "features": ["chat_feature_2"]
                },
            },
        });

        let actual = serde_json::to_value(
            tariff.to_tariff_resource(BTreeSet::<ModuleFeatureId>::new(), module_features),
        )
        .unwrap();

        assert_eq!(actual, expected);
    }
}

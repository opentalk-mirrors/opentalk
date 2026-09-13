// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use opentalk_types_common::{
    features::ModuleFeatureId, modules::ModuleId, tariffs::QuotaType, time::Timestamp,
};

/// Representation of an update to a [`super::Tariff`] in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateTariff {
    /// The name of the tariff.
    pub name: Option<String>,

    /// The update timestamp.
    pub updated_at: Timestamp,

    /// The quotas for the tariff.
    pub quotas: Option<BTreeMap<QuotaType, u64>>,

    /// The list of disabled modules in the tariff.
    pub disabled_modules: Option<Vec<ModuleId>>,

    /// The list of disabled features in the tariff.
    pub disabled_features: Option<Vec<ModuleFeatureId>>,
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use core::fmt::Debug;
use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use opentalk_inventory as inventory;
use opentalk_types_common::{features::ModuleFeatureId, modules::ModuleId, tariffs::QuotaType};

use crate::{schema::tariffs, utils::Jsonb};

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = tariffs)]
pub struct UpdateTariff {
    pub name: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub quotas: Option<Jsonb<BTreeMap<QuotaType, u64>>>,
    pub disabled_modules: Option<Vec<ModuleId>>,
    pub disabled_features: Option<Vec<ModuleFeatureId>>,
}

impl From<inventory::UpdateTariff> for UpdateTariff {
    fn from(
        inventory::UpdateTariff {
            name,
            updated_at,
            quotas,
            disabled_modules,
            disabled_features,
        }: inventory::UpdateTariff,
    ) -> Self {
        Self {
            name,
            updated_at: updated_at.into(),
            quotas: quotas.map(Jsonb),
            disabled_modules,
            disabled_features,
        }
    }
}

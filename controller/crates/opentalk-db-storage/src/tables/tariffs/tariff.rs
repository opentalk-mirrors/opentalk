// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use core::fmt::Debug;
use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use opentalk_inventory as inventory;
use opentalk_types_common::{
    features::ModuleFeatureId,
    modules::ModuleId,
    tariffs::{QuotaType, TariffId},
};
use redis_args::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

use crate::{schema::tariffs, utils::Jsonb};

#[derive(
    Debug,
    Clone,
    Queryable,
    Identifiable,
    Serialize,
    Deserialize,
    ToRedisArgs,
    FromRedisValue,
    PartialEq,
    Eq,
)]
#[to_redis_args(serde)]
#[from_redis_value(serde)]
#[diesel(table_name = tariffs)]
pub struct Tariff {
    pub id: TariffId,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub quotas: Jsonb<BTreeMap<QuotaType, u64>>,
    pub disabled_modules: Vec<Option<ModuleId>>,
    pub disabled_features: Vec<Option<ModuleFeatureId>>,
}

impl From<Tariff> for inventory::Tariff {
    fn from(
        Tariff {
            id,
            name,
            created_at,
            updated_at,
            quotas,
            disabled_modules,
            disabled_features,
        }: Tariff,
    ) -> Self {
        Self {
            id,
            name,
            created_at: created_at.into(),
            updated_at: updated_at.into(),
            quotas: quotas.0,
            disabled_modules,
            disabled_features,
        }
    }
}

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use core::fmt::Debug;

use diesel::prelude::*;
use opentalk_inventory as inventory;
use opentalk_types_common::tariffs::TariffId;

use crate::{schema::external_tariffs, tables::tariffs::ExternalTariffId};

#[derive(Debug, Clone, Insertable, Identifiable, Queryable)]
#[diesel(primary_key(external_id))]
pub struct ExternalTariff {
    pub external_id: ExternalTariffId,
    pub tariff_id: TariffId,
}

impl From<ExternalTariff> for inventory::ExternalTariffMapping {
    fn from(
        ExternalTariff {
            external_id,
            tariff_id,
        }: ExternalTariff,
    ) -> Self {
        Self {
            external_id: external_id.into(),
            tariff_id,
        }
    }
}
